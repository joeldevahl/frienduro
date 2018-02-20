extern crate simplenduro;
extern crate getopts;
extern crate postgis;
extern crate postgres;
extern crate chrono;

use getopts::Options;
use std::env;
use postgis::ewkb;
use chrono::prelude::*;
use self::simplenduro::establish_connection;

struct SegmentMatch {
    pub elapsed: i64,
}

struct SegmentInfo {
    pub sid: i64,
    pub matches: Vec<SegmentMatch>,
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn interp_point(db: &postgres::Connection, rid: i64, point: &ewkb::Point) -> DateTime<Utc> {
    // TODO: asumes we only passes once around the segment
    let rows = db.query(
        "SELECT ts
         FROM points
         WHERE route_id = $1
         ORDER BY ST_Distance(geom, $2) ASC
         LIMIT 1",
        &[&rid, &point],
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("p", "pid", "participation id", "PID");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let pid_str = matches.opt_str("p");

    if pid_str == None {
        print_usage(&program, opts);
        return;
    }

    let db = establish_connection();

    // TODO: to this whole thing in the DB
    let pid: i64 = pid_str.unwrap().parse().unwrap();
    let participation_rows = db.query("SELECT * FROM participations WHERE id = $1", &[&pid])
        .unwrap();

    let rid: i64 = participation_rows.get(0).get("route_id");
    let eid: i64 = participation_rows.get(0).get("event_id");

    // TODO: handle the case where the user submits many atempts on a single event
    let count_rows = db.query(
        "SELECT COUNT(segment_id) FROM participation_segments WHERE participation_id = $1",
        &[&pid],
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
                                 &[&eid])
        .unwrap();

    let mut matched_segments: Vec<SegmentInfo> = Vec::new();

    for row in &segment_rows {
        let sid: i64 = row.get("id");
        let segment_rid: i64 = row.get("route_id");

        let mut segment_info = SegmentInfo {
            sid,
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
                             &[&rid, &segment_rid],
        ).unwrap();

        let segment: ewkb::LineString = matched_rows.get(0).get("segment");
        let segment_start: ewkb::Point = matched_rows.get(0).get("segment_start");
        let segment_end: ewkb::Point = matched_rows.get(0).get("segment_end");

        let is_mls: Option<postgres::Result<ewkb::MultiLineString>> =
            matched_rows.get(0).get_opt("cut");
        match is_mls {
            None => (),
            Some(Ok(mls)) => {
                for ls in mls.lines {
                    match match_segment(&db, ls, &segment_start, &segment_end, pid, rid, sid) {
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
                match match_segment(&db, ls, &segment_start, &segment_end, pid, rid, sid) {
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
            &[&total_elapsed, &pid],
        ).unwrap();

        println!(
            "Matched all segments for a total time of {} seconds",
            total_elapsed
        );
    } else {
        println!(
            "Failed to match all segments ({} of {}). Not enough to qualify for a finished participation",
            total_valid,
            segment_rows.len()
        );
    }
}
