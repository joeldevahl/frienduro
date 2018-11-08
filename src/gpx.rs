use elementtree::Element;

use std::fs::File;
use std::io;
use std::io::prelude::*;

use chrono::prelude::*;

pub fn read_whole_file(filename: &str) -> Result<String, io::Error> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    return Ok(contents);
}

pub struct Point {
    pub lat: f64,
    pub lon: f64,
    pub ele: f64,
    pub utc: DateTime<Utc>,
}

pub fn parse_gpx(gpx_data: &String) -> Result<Vec<Point>, io::Error> {
    let mut points = Vec::new();

    let gpx = Element::from_reader(&mut gpx_data.as_bytes()).unwrap();
    let ns = "http://www.topografix.com/GPX/1/1";
    let trk = gpx.find((ns, "trk")).unwrap();
    let trkseg = trk.find((ns, "trkseg")).unwrap();
    for trkpt in trkseg.find_all((ns, "trkpt")) {
        let mut lat: f64 = 0.0;
        let mut lon: f64 = 0.0;
        let mut ele: f64 = 0.0; // TODO: default elevation?
        let mut utc: DateTime<Utc> = Utc::now(); // TODO: default time?

        match trkpt.get_attr("lat") {
            Some(val) => lat = val.parse().unwrap(),
            None => (),
        }

        match trkpt.get_attr("lon") {
            Some(val) => lon = val.parse().unwrap(),
            None => (),
        }

        match trkpt.find((ns, "ele")) {
            Some(val) => ele = val.text().parse().unwrap(),
            None => (),
        }

        match trkpt.find((ns, "time")) {
            Some(val) => utc = val.text().parse().unwrap(),
            None => (),
        }

        points.push(Point { lat, lon, ele, utc });
    }

    return Ok(points);
}
