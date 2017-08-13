
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

    let segment_rows = db.query("SELECT * FROM segments WHERE event_id = $1", &[&eid])
        .unwrap();

    for row in &segment_rows {
        // TODO: 
/*SELECT
 ST_Intersection(ST_Buffer(segment.route, 1.0), participation.route)
FROM
 (SELECT ST_MakeLine(point) AS route FROM points WHERE route_id = 11) AS participation,
 (SELECT ST_MakeLine(point) AS route FROM points WHERE route_id = 1) AS segment;*/
    }

}
