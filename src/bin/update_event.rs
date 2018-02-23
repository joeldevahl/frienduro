extern crate simplenduro;
extern crate getopts;
extern crate postgis;
extern crate postgres;
extern crate chrono;

use getopts::Options;
use std::env;
use self::simplenduro::establish_connection;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("e", "eid", "event id", "PID");
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

    let event_name: String = event_rows.get(0).get("name");
    println!("Results for event {} ({}):", event_name, eid);

    let event_rows = db.query(
        "SELECT * FROM participations INNER JOIN users ON participations.event_id = $1 AND users.id = participations.user_id ORDER BY participations.total_elapsed_seconds ASC",
        &[&eid],
    ).unwrap();
    for (i, event) in event_rows.into_iter().enumerate() {
        let username: String = event.get("name");
        let maybe_elapsed: Option<postgres::Result<i64>> = event.get_opt("total_elapsed_seconds");
        match maybe_elapsed {
            Some(Ok(elapsed)) => println!("{} - {} {}s", i + 1, username, elapsed),
            Some(Err(..)) | None => println!("{} - {} DNF", i + 1, username),
        }
    }
}
