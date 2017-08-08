
extern crate simplenduro;
extern crate getopts;
extern crate postgis;

use getopts::Options;
use std::env;
use postgis::ewkb;
use self::simplenduro::establish_connection;
use self::simplenduro::gpx;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main()
{
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("p", "pid", "participation id", "PID");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
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

    let pid: i32 = pid_str.unwrap().parse().unwrap();
    let match_rows = db.query("
SELECT
 matched_route,
 ST_3DClosestPoint(participation_route, ST_StartPoint(segment_route)) AS start_point,
 ST_3DClosestPoint(participation_route, ST_EndPoint(segment_route)) AS end_point
FROM
(
 SELECT
  participations.route AS participation_route,
  segments.route AS segment_route,
  ST_Intersection(ST_Buffer(segments.route, 1.0), participations.route) AS matched_route
 FROM
  segments
 INNER JOIN participations ON participations.id = $1
 INNER JOIN event_segments ON (event_segments.event_id = participations.event_id AND segments.id = event_segments.segment_id)
) AS matches
",
                 &[&pid]).unwrap();
    for m in &match_rows {
        let segment: ewkb::LineStringZM = m.get(0);
        let start: ewkb::Geometry = m.get(1);
        let end: ewkb::Geometry = m.get(2);
        // TODO: some safety and checking we actually matched this segment!
        println!("{:?} -> {:?}", start, end);
    }
}
