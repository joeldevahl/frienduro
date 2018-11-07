extern crate frienduro;
extern crate getopts;

use getopts::Options;
use std::env;
use self::frienduro::{establish_connection, create_user};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("n", "name", "user name", "NAME");
    opts.optopt("e", "email", "user email", "EMAIL");
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
    let email = matches.opt_str("e");

    if name == None || email == None {
        print_usage(&program, opts);
        return;
    }

    let db = establish_connection();
    let id = create_user(&db, &name.unwrap(), &email.unwrap());
    println!("Created user with ID {}", id);
}
