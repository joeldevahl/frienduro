extern crate frienduro;
extern crate getopts;
extern crate postgis;
extern crate postgres;
extern crate chrono;

use getopts::Options;
use std::env;
use self::frienduro::establish_connection;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("e", "eid", "event id", "EID");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let eid_str = matches.opt_str("e");

    if eid_str == None {
        print_usage(&program, opts);
        return;
    }

    let db = establish_connection();

    // TODO: to this whole thing in the DB
    let eid: i64 = eid_str.unwrap().parse().unwrap();
    let event_rows = db.query("SELECT * FROM events WHERE id = $1", &[&eid])
        .unwrap();
    let participation_rows = db.query("SELECT * FROM participations WHERE (event_id = $1 AND total_elapsed_seconds IS NOT NULL) ORDER BY total_elapsed_seconds DESC", &[&eid])
        .unwrap();

    let event_name: String = event_rows.get(0).get("name");
    println!("Results for {}:", event_name);
    for (i, participation_row) in participation_rows.iter().enumerate() {
        let uid: i64 = participation_row.get("user_id");
        let seconds: i64 = participation_row.get("total_elapsed_seconds");
        println!("{} UID {} - {} seconds", i + 1, uid, seconds);
    }
}
