extern crate dotenv;
extern crate gpx;
extern crate postgres;
extern crate postgis;
extern crate chrono;

use dotenv::dotenv;
use std::env;
use chrono::prelude::*;

use postgres::{Connection, TlsMode};
use postgis::ewkb;

use std::io::prelude::*;
use std::fs::File;

pub fn read_whole_file(path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    return Ok(contents);
}

pub fn read_gpx(gpx_data: &str) -> Result<gpx::Gpx, gpx::errors::Error> {
    let reader = std::io::Cursor::new(gpx_data.as_bytes());

    gpx::read(reader)
}

pub fn establish_connection() -> Connection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap();
    Connection::connect(database_url, TlsMode::None).unwrap()
}

const EMPTY_DB_SQL: &'static str = include_str!("empty_db.sql");
const CREATE_DB_SQL: &'static str = include_str!("create_db.sql");

pub fn create_db(db: &Connection) -> Result<(), postgres::Error> {
    match db.batch_execute(EMPTY_DB_SQL) {
        Ok(_) => (),
        Err(_) => (),
    }

    db.batch_execute(CREATE_DB_SQL)
}

pub fn empty_db(db: &Connection) -> Result<(), postgres::Error> {
    db.batch_execute(EMPTY_DB_SQL)
}

pub fn create_user(db: &Connection, name: &str, email: &str) -> i64 {
    let rows = db.query(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
        &[&name, &email],
    ).unwrap();
    rows.get(0).get(0)
}

pub fn create_source_route(db: &Connection, gpx_data: &str) -> i64 {
    let rows = db.query(
        "INSERT INTO source_routes (gpx) VALUES (XMLPARSE (DOCUMENT $1)) RETURNING id",
        &[&gpx_data],
    ).unwrap();
    rows.get(0).get(0)
}

pub fn get_new_route_id(db: &Connection) -> i64 {
    let rows = db.query("SELECT nextval('route_id_seq')", &[]).unwrap();
    rows.get(0).get(0)
}

pub fn store_points(db: &Connection, route_id: i64, points: &[gpx::Waypoint]) {
    for p in points {
        let point = ewkb::Point {
            x: p.point().x(),
            y: p.point().y(),
            srid: Some(4326),
        };
        db.execute(
            "INSERT INTO points (geom, route_id, ts, ele) VALUES ($1, $2, $3, $4)",
            &[&point, &route_id, &p .time.unwrap(), &p.elevation.unwrap()],
        ).unwrap();
    }
}

pub fn create_segment(db: &Connection, name: &str, route_id: i64, source_id: i64) -> i64 {
    let rows = db.query("INSERT INTO segments (name, route_id, source_id) VALUES ($1, $2, $3) RETURNING id",
        &[&name, &route_id, &source_id]
    ).unwrap();
    let segment_id: i64 = rows.get(0).get(0);

    db.execute(
        "UPDATE segments SET geom = line.geom, geom_expanded = ST_Buffer(line.geom, 20, 'endcap=flat join=round')
        FROM (SELECT ST_MakeLine(geom::geometry)::geography AS geom FROM points WHERE route_id = $1) AS line
        WHERE id = $2",
        &[&route_id, &source_id],
    ).unwrap();

    segment_id
}

pub fn create_event(db: &Connection, name: &str, segment_ids: &[i64]) -> i64{
    let rows = db.query(
        "INSERT INTO events (name) VALUES ($1) RETURNING id",
        &[&name],
    ).unwrap();
    let event_id: i64 = rows.get(0).get(0);

    for segment_id in segment_ids {
        db.execute(
            "INSERT INTO event_segments (event_id, segment_id) VALUES ($1, $2)",
            &[&event_id, &segment_id],
        ).unwrap();
    }

    event_id
}

struct SegmentMatch {
    pub elapsed: i64,
}

struct SegmentInfo {
    pub segment_id: i64,
    pub matches: Vec<SegmentMatch>,
}

fn interp_point(db: &postgres::Connection, route_id: i64, point: &ewkb::Point) -> DateTime<Utc> {
    // TODO: assumes we only passes once around the segment
    let rows = db.query(
        "SELECT ts
         FROM points
         WHERE route_id = $1
         ORDER BY ST_Distance(geom, $2) ASC
         LIMIT 1",
        &[&route_id, &point],
    ).unwrap();
    return rows.get(0).get(0);
}

fn match_segment(
    db: &postgres::Connection,
    ls: ewkb::LineString,
    segment_start: &ewkb::Point,
    segment_end: &ewkb::Point,
    pid: i64,
    rid: i64,
    sid: i64,
) -> Option<chrono::Duration> {
    let points = &ls.points;
    let start = &points[0];
    let end = &(points.last().unwrap());

    let distance_rows = db.query(
        "SELECT
        ST_Distance($1::geography, $2::geography) AS dist_start,
        ST_Distance($3::geography, $4::geography) AS dist_end",
        &[&segment_start, &start, &segment_end, &end],
    ).unwrap();

    let distance_start: f64 = distance_rows.get(0).get(0);
    let distance_end: f64 = distance_rows.get(0).get(1);
    if distance_start < 20.0 && distance_end < 20.0 {
        let start_time = interp_point(&db, rid, start);
        let end_time = interp_point(&db, rid, end);
        let diff = end_time.signed_duration_since(start_time);

        if diff >= chrono::Duration::seconds(0) {
            let seconds: i64 = diff.num_seconds();
            db.execute(
                "INSERT INTO participation_segments (participation_id, segment_id, elapsed_seconds, geom) VALUES ($1, $2, $3, $4)",
                &[&pid, &sid, &seconds, &ls],
            ).unwrap();

            return Some(diff);
        }
    }

    return None;
}

fn update_participation_timing(db: &Connection, participation_id: i64) {
    // TODO: to this whole thing in the DB
    let participation_rows = db.query("SELECT * FROM participations WHERE id = $1", &[&participation_id])
        .unwrap();

    let route_id: i64 = participation_rows.get(0).get("route_id");
    let event_id: i64 = participation_rows.get(0).get("event_id");

    // TODO: handle the case where the user submits many atempts on a single event
    let count_rows = db.query(
        "SELECT COUNT(segment_id) FROM participation_segments WHERE participation_id = $1",
        &[&participation_id],
    ).unwrap();
    let old_count: i64 = count_rows.get(0).get(0);
    if old_count > 0 {
        println!(
            "Participation already has all data set! Need to implement multi attempt support..."
        );
        return;
    }

    let segment_rows = db.query("SELECT
                                    *
                                 FROM
                                    segments
                                 INNER JOIN
                                    event_segments
                                 ON (event_segments.event_id = $1 AND segments.id = event_segments.segment_id)",
                                 &[&event_id])
        .unwrap();

    let mut matched_segments: Vec<SegmentInfo> = Vec::new();

    for row in &segment_rows {
        let segment_id: i64 = row.get("id");
        let segment_route_id: i64 = row.get("route_id");

        let mut segment_info = SegmentInfo {
            segment_id,
            matches: Vec::new(),
        };

        let matched_rows = db.query("SELECT
                                        ST_Intersection(ST_Buffer(segment.route, 20, 'endcap=flat join=round'), participation.route) AS cut,
                                        segment.route AS segment,
                                        ST_StartPoint(segment.route::geometry) AS segment_start,
                                        ST_EndPoint(segment.route::geometry) AS segment_end,
                                        participation.route AS participation
                                    FROM
                                    (SELECT ST_MakeLine(geom::geometry)::geography AS route FROM points WHERE route_id = $1) AS participation,
                                    (SELECT ST_MakeLine(geom::geometry)::geography AS route FROM points WHERE route_id = $2) AS segment",
                             &[&route_id, &segment_route_id],
        ).unwrap();

        let segment_start: ewkb::Point = matched_rows.get(0).get("segment_start");
        let segment_end: ewkb::Point = matched_rows.get(0).get("segment_end");

        let is_mls: Option<postgres::Result<ewkb::MultiLineString>> =
            matched_rows.get(0).get_opt("cut");
        match is_mls {
            None => (),
            Some(Ok(mls)) => {
                for ls in mls.lines {
                    match match_segment(&db, ls, &segment_start, &segment_end, participation_id, route_id, segment_id) {
                        Some(seconds) => {
                            let segment_match = SegmentMatch { elapsed: seconds.num_seconds() };
                            segment_info.matches.push(segment_match);
                        }
                        None => (),
                    }
                }
            }
            Some(Err(..)) => {
                let ls: ewkb::LineString = matched_rows.get(0).get("cut");
                match match_segment(&db, ls, &segment_start, &segment_end, participation_id, route_id, segment_id) {
                    Some(seconds) => {
                        let segment_match = SegmentMatch { elapsed: seconds.num_seconds() };
                        segment_info.matches.push(segment_match);
                    }
                    None => (),
                }
            }
        }

        matched_segments.push(segment_info);
    }

    // TODO: more advanced completion logic
    // for now we just make sure all segments are matched, and take the fastest time
    let mut total_elapsed: i64 = 0;
    let mut total_valid: usize = 0;
    for segment_info in matched_segments {
        let valid = segment_info.matches.len() != 0;
        let mut smallest: i64 = std::i64::MAX;
        for segment_match in segment_info.matches {
            if segment_match.elapsed < smallest {
                smallest = segment_match.elapsed;
            }
        }

        if valid {
            total_elapsed += smallest;
            total_valid += 1;
        }
    }

    if total_valid == segment_rows.len() {
        // TODO: update this from DB instead of from here
        db.execute(
            "UPDATE participations SET total_elapsed_seconds = $1
            WHERE id = $2",
            &[&total_elapsed, &participation_id],
        ).unwrap();
    }
}

pub fn create_participation(db: &Connection, event_id: i64, user_id: i64, route_id: i64, source_id: i64) -> i64 {
    let rows = db.query("INSERT INTO participations (event_id, user_id, route_id, source_id) VALUES ($1, $2, $3, $4) RETURNING id",
        &[&event_id, &user_id, &route_id, &source_id]
    ).unwrap();
    let participation_id: i64 = rows.get(0).get(0);

    db.execute(
        "UPDATE participations SET geom = line.geom
        FROM (SELECT ST_MakeLine(geom::geometry)::geography AS geom FROM points WHERE route_id = $1) AS line
        WHERE id = $2",
        &[&route_id, &participation_id],
    ).unwrap();

    update_participation_timing(db, participation_id);

    participation_id
}

pub struct EventResult {
    pub username: String,
    pub time: i64,
}

pub fn get_event_results(db: &Connection, event_id: i64) -> Vec<EventResult> {
    let event_rows = db.query(
        "SELECT * FROM participations INNER JOIN users ON participations.event_id = $1 AND users.id = participations.user_id ORDER BY participations.total_elapsed_seconds ASC",
        &[&event_id],
    ).unwrap();

    event_rows.iter().map(|row| {
        let username: String = row.get("name");
        let maybe_elapsed: Option<postgres::Result<i64>> = row.get_opt("total_elapsed_seconds");
        let time = match maybe_elapsed {
            Some(Ok(elapsed)) => elapsed,
            Some(Err(..)) | None => 0,
        };

        EventResult { username, time }
    }).collect()
}