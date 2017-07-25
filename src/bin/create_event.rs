extern crate simplenduro;
extern crate getopts;

use getopts::Options;
use std::env;
use self::simplenduro::establish_connection;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main()
{
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("n", "name", "event name", "NAME");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let name = matches.opt_str("n");

    if name == None || matches.free.is_empty() {
        print_usage(&program, opts);
        return;
    }

    let db = establish_connection();
    let rows = db.query("INSERT INTO events (name) VALUES ($1) RETURNING id",
                 &[&name.unwrap()]).unwrap();
    let id: i32 = rows.get(0).get(0);

    for segment_id in matches.free {
        let sid: i32 = segment_id.parse().unwrap();
        db.execute("INSERT INTO event_segments (event_id, segment_id) VALUES ($1, $2)",
                 &[&id, &sid]).unwrap();
    }

    println!("Created event with ID {}", id);
}
