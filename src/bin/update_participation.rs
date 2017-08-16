
extern crate simplenduro;
extern crate getopts;
extern crate postgis;
extern crate geo;

use getopts::Options;
use std::env;
use postgis::ewkb;
use geo::Point;
use geo::algorithm::haversine_distance::HaversineDistance;
use self::simplenduro::establish_connection;
use self::simplenduro::gpx;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
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
    let pid: i32 = pid_str.unwrap().parse().unwrap();
    let participation_rows = db.query("SELECT * FROM participations WHERE id = $1", &[&pid])
        .unwrap();

    let rid: i32 = participation_rows.get(0).get("route_id");
    let eid: i32 = participation_rows.get(0).get("event_id");

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
        let segment_rid: i32 = row.get("route_id");

        let matched_rows = db.query("SELECT
                                        ST_Intersection(ST_Buffer(segment.route, 1.0), participation.route) AS cut,
                                        segment.route AS segment,
                                        ST_StartPoint(segment.route) AS segment_start,
                                        ST_EndPoint(segment.route) AS segment_end,
                                        participation.route AS participation
                                    FROM
                                    (SELECT ST_MakeLine(geom) AS route FROM points WHERE route_id = $1) AS participation,
                                    (SELECT ST_MakeLine(geom) AS route FROM points WHERE route_id = $2) AS segment",
                             &[&rid, &segment_rid],
        ).unwrap();

        let mls: ewkb::MultiLineString = matched_rows.get(0).get("cut");
        let segment: ewkb::LineString = matched_rows.get(0).get("segment");
        let segment_start: ewkb::Point = matched_rows.get(0).get("segment_start");
        let segment_end: ewkb::Point = matched_rows.get(0).get("segment_end");

        for ls in mls.lines {
            let points = ls.points;
            let start = &points[0];
            let end = &(points.last().unwrap());

            let ds = Point::new(start.y, start.x)
                .haversine_distance(&Point::new(segment_start.y, segment_start.x));
            let de = Point::new(end.y, end.x)
                .haversine_distance(&Point::new(segment_end.y, segment_end.x));

            println!("Match with ds {} and de {}", ds, de);
        }
    }
}
