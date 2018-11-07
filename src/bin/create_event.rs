extern crate frienduro;
extern crate getopts;

use getopts::Options;
use std::env;
use self::frienduro::{establish_connection, create_event};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] SEGMENT_IDS...", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("n", "name", "event name", "NAME");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
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

    let segment_ids: Vec<i64> = matches.free.iter().map(|x| x.parse().unwrap()).collect();

    let db = establish_connection();
    let event_id = create_event(&db, &name.unwrap(), &segment_ids);
    
    println!("Created event with ID {}", event_id);
}
