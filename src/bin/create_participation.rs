extern crate frienduro;
extern crate getopts;
extern crate postgis;

use getopts::Options;
use std::env;
use postgis::ewkb;
use self::frienduro::establish_connection;
use self::frienduro::gpx;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("u", "uid", "user id", "UID");
    opts.optopt("e", "eid", "event id", "EID");
    opts.optopt("g", "gpx", "GPX file", "FILE");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
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

    let uid: i64 = uid_str.unwrap().parse().unwrap();
    let eid: i64 = eid_str.unwrap().parse().unwrap();

    let gpx_data = gpx::read_whole_file(file.unwrap()).unwrap();
    let source_rows = db.query(
        "INSERT INTO source_routes (gpx) VALUES (XMLPARSE (DOCUMENT $1)) RETURNING id",
        &[&gpx_data],
    ).unwrap();
    let source_id: i64 = source_rows.get(0).get(0);

    let part_rows = db.query("INSERT INTO participations (event_id, user_id, route_id, source_id) VALUES ($1, $2, nextval('route_id_seq'), $3) RETURNING id, route_id",
                 &[&eid, &uid, &source_id]).unwrap();
    let pid: i64 = part_rows.get(0).get(0);
    let rid: i64 = part_rows.get(0).get(1);

    let points = gpx::parse_gpx(gpx_data).unwrap();
    for point in points {
        let p = ewkb::Point {
            x: point.lon,
            y: point.lat,
            srid: Some(4326),
        };
        db.execute(
            "INSERT INTO points (geom, route_id, ts, ele) VALUES ($1, $2, $3, $4)",
            &[&p, &rid, &point.utc, &point.ele],
        ).unwrap();
    }

    db.execute(
        "UPDATE participations SET geom = line.geom
        FROM (SELECT ST_MakeLine(geom::geometry)::geography AS geom FROM points WHERE route_id = $1) AS line
        WHERE id = $2",
        &[&rid, &pid],
    ).unwrap();

    println!("Created participation with ID {}", pid);
}
