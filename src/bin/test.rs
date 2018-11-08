extern crate frienduro;
extern crate postgres;
extern crate postgis;

use self::frienduro::*;
use self::frienduro::gpx::*;
use postgres::Connection;
use postgis::ewkb;

fn create_segments_helper(db: &Connection, name: &str, filename: &str, splits: usize, pad: f32) -> Vec<i64> {
    let gpx_data = gpx::read_whole_file(filename).unwrap();
    let source_id = create_source_route(&db, &gpx_data);
    let points = gpx::parse_gpx(&gpx_data).unwrap();

    let tot_pad = pad * ((splits + 3) as f32);
    let split_len = (1.0 - tot_pad) / ((splits + 1) as f32);

    let num_positions = points.len();
    let samples_per_split = ((num_positions as f32) * split_len) as usize;
    let samples_per_pad = ((num_positions as f32) * pad) as usize;

    (0..splits+1).map(|s| {
        let start = s * (samples_per_split + samples_per_pad) + samples_per_pad;
        let end = start + samples_per_split;

        let route_id = get_new_route_id(&db);
        store_points(&db, route_id, &points[start..end]);

        let segment_name = format!("{} ({})", name, s);
        create_segment(&db, &segment_name, route_id, source_id)
    }).collect::<Vec<i64>>()
}

fn create_participation_helper(db: &Connection, event_id: i64, user_id: i64, filename: &str) -> i64 {
    let gpx_data = gpx::read_whole_file(filename).unwrap();
    let source_id = create_source_route(&db, &gpx_data);
    let points = gpx::parse_gpx(&gpx_data).unwrap();
    let route_id = get_new_route_id(&db);
    store_points(&db, route_id, &points);
    create_participation(&db, event_id, user_id, route_id, source_id)
}

fn main() {
    let db = establish_connection();
    create_db(&db).unwrap();
    let john = create_user(&db, "John Doe", "john@doe.org");
    let jane = create_user(&db, "Jane Doe", "jane@doe.org");
    let segments = create_segments_helper(&db, "Foo Segment", "gpx\\Harnon_Runt_2017_joel.gpx", 9, 0.05);
    let event_id = create_event(&db, "Test Race", &segments);
    let john_participation = create_participation_helper(&db, event_id, john, "gpx\\Harnon_Runt_2017_joel.gpx");
    let jane_participation = create_participation_helper(&db, event_id, jane, "gpx\\Harnon_Runt_2017_marika.gpx");
    let results = get_event_results(&db, event_id);
    for (i, result) in results.iter().enumerate() {
        match result.time {
            0 => println!("{} - {} DNF", i + 1, result.username),
            time => println!("{} - {} {}s", i + 1, result.username, time),
        }
    }
}
