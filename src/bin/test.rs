extern crate frienduro;
extern crate getopts;
extern crate postgis;
extern crate postgres;

use self::frienduro::*;
use getopts::Options;
use std::env;
use std::fs;
use std::path::Path;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") || matches.free.is_empty() {
        print_usage(&program, opts);
        return;
    }

    let db = establish_connection();
    for event_dir in matches.free {
        create_db(&db).unwrap(); // TODO: persist DB

        let event_path = Path::new(&event_dir);
        let event_name = event_path.file_name().unwrap().to_str().unwrap();
        println!("Processing event: {}", event_name);

        let segments_path = event_path.join("segments");
        let segment_files = fs::read_dir(segments_path).unwrap();
        let segment_ids = segment_files
            .into_iter()
            .map(|f| {
                let segment_file = f.unwrap();
                let segment_file_name = segment_file.file_name();
                let segment_name = segment_file_name.to_str().unwrap();
                println!("\tadding segment: {}", segment_name);

                let path = segment_file.path();
                let filename = path.to_str().unwrap();
                let gpx_data = read_whole_file(filename).unwrap();
                let source_id = create_source_route(&db, &gpx_data).unwrap();

                let gpx = read_gpx(&gpx_data).unwrap();
                let track = &gpx.tracks[0];
                let segment = &track.segments[0];
                let points = &segment.points;
                let route_id = get_new_route_id(&db).unwrap();
                store_points(&db, route_id, &points);

                create_segment(&db, segment_name, route_id, source_id).unwrap()
            })
            .collect::<Vec<i64>>();
        println!();

        let event_id = create_event(&db, event_name, &segment_ids);

        let users_path = event_path.join("users");
        let users = fs::read_dir(users_path).unwrap();
        for u in users {
            let user_path = u.unwrap().path();
            let ext = user_path.extension().unwrap();
            if ext == "gpx" {
                let user_name = user_path.file_stem().unwrap().to_str().unwrap();
                let user_id = create_user(&db, user_name, "").unwrap();
                println!("\tadding user: {}", user_name);

                let filename = user_path.to_str().unwrap();
                let gpx_data = read_whole_file(filename).unwrap();
                let source_id = create_source_route(&db, &gpx_data).unwrap();
                let gpx = read_gpx(&gpx_data).unwrap();
                let track = &gpx.tracks[0];
                let segment = &track.segments[0];
                let points = &segment.points;
                let route_id = get_new_route_id(&db).unwrap();
                store_points(&db, route_id, &points);
                create_participation(&db, event_id, user_id, route_id, source_id);
            }
        }
        println!();

        println!("\tresults:");
        let results = get_event_results(&db, event_id);
        for (i, result) in results.iter().enumerate() {
            match result.time {
                0 => println!("\t\t{} - {} DNF", i + 1, result.username),
                time => println!("\t\t{} - {} {}s", i + 1, result.username, time),
            }
        }
    }
}
