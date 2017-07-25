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
    let ls = gpx::parse_gpx(file.unwrap()).unwrap();
    let rows = db.query("INSERT INTO participations (event_id, user_id, route) VALUES ($1, $2, $3) RETURNING id",
                 &[&eid, &uid, &ls]).unwrap();
    let id: i32 = rows.get(0).get(0);
    println!("Created participation with ID {}", id);
}
