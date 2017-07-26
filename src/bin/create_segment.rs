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
    opts.optopt("n", "name", "segment name", "NAME");
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
    let name = matches.opt_str("n");
    let file = matches.opt_str("g");

    if name == None || file == None {
        print_usage(&program, opts);
        return;
    }

    let db = establish_connection();

    let gpx_data = gpx::read_whole_file(file.unwrap()).unwrap();
    let source_rows = db.query("INSERT INTO source_routes (gpx) VALUES (XMLPARSE (DOCUMENT $1)) RETURNING id",
                 &[&gpx_data]).unwrap();
    let source_id: i32 = source_rows.get(0).get(0);

    let ls = gpx::parse_gpx(gpx_data).unwrap();
    let segment_rows = db.query("INSERT INTO segments (name, route, source_id) VALUES ($1, $2, $3) RETURNING id",
                 &[&name.unwrap(), &ls, &source_id]).unwrap();
    let segment_id: i32 = segment_rows.get(0).get(0);
    println!("Created segment with ID {}", segment_id);
}
