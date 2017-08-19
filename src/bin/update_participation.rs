
extern crate simplenduro;
extern crate getopts;
extern crate postgis;
extern crate postgres;

use getopts::Options;
use std::env;
use postgis::ewkb;
use self::simplenduro::establish_connection;
use self::simplenduro::gpx;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}
/*
fn interp_point() {

    let start_points_rows = db.query(
        "SELECT
            *
         FROM
            points
         WHERE
            route_id = $1"
        &[&segment_start, &start, &segment_end, &end],
    ).unwrap();

}*/

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

    let segment_rows = db.query("SELECT
                                    *
                                 FROM
                                    segments
                                 INNER JOIN
                                    event_segments
                                 ON (event_segments.event_id = $1 AND segments.id = event_segments.segment_id)",
                                 &[&eid])
        .unwrap();

    for row in &segment_rows {
        let sid: i64 = row.get("id");
        let segment_rid: i64 = row.get("route_id");
        println!("Segment {}", sid);

        let matched_rows = db.query("SELECT
                                        ST_Intersection(ST_Buffer(segment.route, 2, 'endcap=flat join=round'), participation.route) AS cut,
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
            Some(Ok(mls)) => for ls in mls.lines {
                let points = ls.points;
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
                if distance_start < 2.0 && distance_end < 2.0 {
                    println!(
                        "\tMatch {} -> {} ({:?}, {:?})",
                        distance_start,
                        distance_end,
                        start,
                        end
                    );
                    break;
                }
            },
            Some(Err(err)) => {
                let ls: ewkb::LineString = matched_rows.get(0).get("cut");

                let points = ls.points;
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
                if distance_start < 2.0 && distance_end < 2.0 {

                    println!(
                        "\tMatch {} -> {} ({:?}, {:?})",
                        distance_start,
                        distance_end,
                        start,
                        end
                    );
                }
            }
        }
    }
}
