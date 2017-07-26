extern crate simplenduro;
extern crate getopts;

use getopts::Options;
use std::env;
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
    opts.optopt("u", "uid", "user id", "UID");
    opts.optopt("e", "eid", "event id", "EID");
    opts.optopt("g", "gpx", "GPX file", "FILE");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let uid_str = matches.opt_str("u");
    let eid_str = matches.opt_str("e");
    let file = matches.opt_str("g");

    if uid_str == None || eid_str == None || file == None {
        print_usage(&program, opts);
        return;
    }

    let db = establish_connection();

    let uid: i32 = uid_str.unwrap().parse().unwrap();
    let eid: i32 = eid_str.unwrap().parse().unwrap();

    let gpx_data = gpx::read_whole_file(file.unwrap()).unwrap();
    let source_rows = db.query("INSERT INTO source_routes (gpx) VALUES (XMLPARSE (DOCUMENT $1)) RETURNING id",
                 &[&gpx_data]).unwrap();
    let source_id: i32 = source_rows.get(0).get(0);

    let ls = gpx::parse_gpx(gpx_data).unwrap();
    let part_rows = db.query("INSERT INTO participations (event_id, user_id, route, source_id) VALUES ($1, $2, $3, $4) RETURNING id",
                 &[&eid, &uid, &ls, &source_id]).unwrap();
    let part_id: i32 = part_rows.get(0).get(0);
    println!("Created participation with ID {}", part_id);
}
