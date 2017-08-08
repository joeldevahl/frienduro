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
    opts.optopt("n", "name", "segment name", "NAME");
    opts.optopt("g", "gpx", "GPX file", "FILE");
    opts.optopt("s", "splits", "times to split file", "NUM_SPLITS");
    opts.optopt("p", "pad", "padding (in percents) between splits", "PAD");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string())
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let name = matches.opt_str("n");
    let file = matches.opt_str("g");
    let splits = match matches.opt_str("s") {
        Some(s) => s.parse().unwrap(),
        None => 0
    };
    let pad = match matches.opt_str("p") {
        Some(p) => p.parse().unwrap(),
        None => 0.0
    } / 100.0;

    let tot_pad = pad * (splits as f32);
    let split_len = (1.0 - tot_pad) / ((splits+1) as f32);

    if name == None || file == None {
        print_usage(&program, opts);
        return;
    }

    let db = establish_connection();

    let gpx_data = gpx::read_whole_file(file.unwrap()).unwrap();
    let source_rows = db.query("INSERT INTO source_routes (gpx) VALUES (XMLPARSE (DOCUMENT $1)) RETURNING id",
                 &[&gpx_data]).unwrap();
    let source_id: i32 = source_rows.get(0).get(0);

    let points = gpx::parse_gpx(gpx_data).unwrap();
    let n = name.unwrap();
    let num_positions = points.len();
    let samples_per_split = ((num_positions as f32) * split_len) as usize;
    let samples_per_pad = ((num_positions as f32) * pad) as usize;

    for s in 0..splits+1 {
        let start = s * (samples_per_split + samples_per_pad);
        let end = start + samples_per_split;

        let segment_points = points[start..end].to_vec();
        let segment_ls = ewkb::LineStringZM{points: segment_points, srid: Some(4326)};
        let segment_name = format!("{} ({})", n, s);
        let segment_rows = db.query("INSERT INTO segments (name, route, source_id) VALUES ($1, $2, $3) RETURNING id",
                     &[&segment_name, &segment_ls, &source_id]).unwrap();
        let segment_id: i32 = segment_rows.get(0).get(0);
        println!("Created segment with name {} and ID {}", segment_name, segment_id);
    }
}
