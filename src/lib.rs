extern crate chrono;
extern crate dotenv;
extern crate gpx;
extern crate postgis;
extern crate postgres;

use chrono::prelude::*;
use dotenv::dotenv;
use std::env;

use postgis::ewkb;
use postgres::{Connection, TlsMode};

use postgis::ewkb::{LineStringZ, PointZ};
use std::fs::File;
use std::io::prelude::*;

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

pub struct User {
    pub name: String,
    pub email: String,
}

pub fn create_user(db: &Connection, name: &str, email: &str) -> Option<i64> {
    match db.query(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
        &[&name, &email],
    ) {
        Ok(rows) => Some(rows.get(0).get(0)),
        Err(_) => None,
    }
}

pub fn get_user(db: &Connection, user_id: i64) -> Option<User> {
    match db.query("SELECT * FROM users WHERE id = $1", &[&user_id]) {
        Ok(rows) => Some(User {
            name: rows.get(0).get("name"),
            email: rows.get(0).get("email"),
        }),
        Err(_) => None,
    }
}

pub fn create_segment(db: &Connection, name: &str, waypoints: &[gpx::Waypoint]) -> Option<i64> {
    let points = waypoints
        .iter()
        .map(|wp| {
            let p = wp.point();
            ewkb::Point {
                x: p.x(),
                y: p.y(),
                srid: Some(4326),
            }
        })
        .collect::<Vec<ewkb::Point>>();
    let line = ewkb::LineString {
        points,
        srid: Some(4326),
    };

    match db.query(
        "INSERT INTO segments (name, geom) VALUES ($1, $2) RETURNING id",
        &[&name, &line],
    ) {
        Ok(rows) => {
            let segment_id: i64 = rows.get(0).get(0);

            // TODO: do this on insert and skip a query
            db.execute(
                "UPDATE segments SET geom_expanded = ST_Buffer(geom, 20, 'endcap=flat join=round') WHERE id = $1",
                &[&segment_id],
            ).unwrap();

            Some(segment_id)
        }
        Err(err) => None,
    }
}

pub fn create_event(db: &Connection, name: &str, segment_ids: &[i64]) -> i64 {
    let rows = db
        .query(
            "INSERT INTO events (name) VALUES ($1) RETURNING id",
            &[&name],
        )
        .unwrap();
    let event_id: i64 = rows.get(0).get(0);

    for segment_id in segment_ids {
        db.execute(
            "INSERT INTO event_segments (event_id, segment_id) VALUES ($1, $2)",
            &[&event_id, &segment_id],
        )
        .unwrap();
    }

    event_id
}

struct SegmentMatch {
    pub elapsed: f64,
}

struct SegmentInfo {
    pub segment_id: i64,
    pub matches: Vec<SegmentMatch>,
}

fn check_dist(p1: &ewkb::Point, p2: &ewkb::PointZ, threshold: f64) -> bool {
    // TODO: this thing is completely bollocks now. Threshold is in meters but points are lat/lon
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    dx * dx + dy * dy <= threshold * threshold
}

fn check_line(line: &ewkb::LineStringZ, start: &ewkb::Point, end: &ewkb::Point) -> Option<f64> {
    let ls = &line.points[0];
    let le = &line.points[line.points.len() - 1];
    match check_dist(start, ls, 20.0) && check_dist(end, le, 20.0) {
        true => Some(le.z - ls.z),
        false => None,
    }
}

fn match_segments(
    db: &postgres::Connection,
    lines: &Vec<ewkb::LineStringZ>,
    segment_start: &ewkb::Point,
    segment_end: &ewkb::Point,
    pid: i64,
    sid: i64,
) -> Option<f64> {
    if lines.len() == 0 {
        return None;
    }

    // TODO: hack to match only first line for now
    check_line(&lines[0], segment_start, segment_end)
}

fn update_participation_timing(db: &Connection, participation_id: i64) {
    // TODO: to this whole thing in the DB
    let participation_rows = db
        .query(
            "SELECT * FROM participations WHERE id = $1",
            &[&participation_id],
        )
        .unwrap();

    let event_id: i64 = participation_rows.get(0).get("event_id");

    // TODO: handle the case where the user submits many attempts on a single event
    let count_rows = db
        .query(
            "SELECT COUNT(segment_id) FROM participation_segments WHERE participation_id = $1",
            &[&participation_id],
        )
        .unwrap();
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

        let mut segment_info = SegmentInfo {
            segment_id,
            matches: Vec::new(),
        };

        let matched_rows = db.query("SELECT
                                        ST_Intersection(segment.geom_expanded, participation.geom) AS cut,
                                        ST_StartPoint(segment.geom::geometry) AS segment_start,
                                        ST_EndPoint(segment.geom::geometry) AS segment_end
                                    FROM
                                    (SELECT geom FROM participations WHERE id = $1) AS participation,
                                    (SELECT geom, geom_expanded FROM segments WHERE id = $2) AS segment",
                             &[&participation_id, &segment_id],
        ).unwrap();

        let segment_start: ewkb::Point = matched_rows.get(0).get("segment_start");
        let segment_end: ewkb::Point = matched_rows.get(0).get("segment_end");

        let is_mls: Option<postgres::Result<ewkb::MultiLineStringZ>> =
            matched_rows.get(0).get_opt("cut");
        let lines = match is_mls {
            Some(Ok(mls)) => mls.lines,
            Some(Err(..)) => {
                let ls: ewkb::LineStringZ = matched_rows.get(0).get("cut");
                vec![ls]
            }
            None => vec![],
        };

        match match_segments(
            &db,
            &lines,
            &segment_start,
            &segment_end,
            participation_id,
            segment_id,
        ) {
            Some(seconds) => {
                let segment_match = SegmentMatch { elapsed: seconds };
                segment_info.matches.push(segment_match);
            }
            None => (),
        }

        matched_segments.push(segment_info);
    }

    // TODO: more advanced completion logic
    // for now we just make sure all segments are matched, and take the fastest time
    let mut total_elapsed: f64 = 0.0;
    let mut total_valid: usize = 0;
    for segment_info in matched_segments {
        let valid = segment_info.matches.len() != 0;
        let mut smallest: f64 = std::f64::MAX;
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
        )
        .unwrap();
    }
}

pub fn create_participation(
    db: &Connection,
    event_id: i64,
    user_id: i64,
    waypoints: &[gpx::Waypoint],
) -> i64 {
    let start_time = waypoints[0].time.unwrap().timestamp_millis();
    let points = waypoints
        .iter()
        .map(|wp| {
            let p = wp.point();
            ewkb::PointZ {
                x: p.x(),
                y: p.y(),
                z: (wp.time.unwrap().timestamp_millis() - start_time) as f64 / 1000.0,
                srid: Some(4326),
            }
        })
        .collect::<Vec<ewkb::PointZ>>();
    let line = ewkb::LineStringZ {
        points,
        srid: Some(4326),
    };

    let rows = db
        .query(
            "INSERT INTO participations (event_id, user_id, geom) VALUES ($1, $2, $3) RETURNING id",
            &[&event_id, &user_id, &line],
        )
        .unwrap();
    let participation_id: i64 = rows.get(0).get(0);

    update_participation_timing(db, participation_id);

    participation_id
}

pub struct EventResult {
    pub username: String,
    pub time: f64,
}

pub fn get_event_results(db: &Connection, event_id: i64) -> Vec<EventResult> {
    let event_rows = db.query(
        "SELECT * FROM participations INNER JOIN users ON participations.event_id = $1 AND users.id = participations.user_id ORDER BY participations.total_elapsed_seconds ASC",
        &[&event_id],
    ).unwrap();

    event_rows
        .iter()
        .map(|row| {
            let username: String = row.get("name");
            let maybe_elapsed: Option<postgres::Result<f64>> = row.get_opt("total_elapsed_seconds");
            let time = match maybe_elapsed {
                Some(Ok(elapsed)) => elapsed,
                Some(Err(..)) | None => 0.0,
            };

            EventResult { username, time }
        })
        .collect()
}

pub struct Event {
    pub name: String,
    pub results: Vec<EventResult>,
}

pub fn get_event(db: &Connection, event_id: i64) -> Option<Event> {
    match db.query("SELECT * FROM events WHERE id = $1", &[&event_id]) {
        Ok(rows) => Some(Event {
            name: rows.get(0).get("name"),
            results: get_event_results(db, event_id),
        }),
        Err(..) => None,
    }
}
