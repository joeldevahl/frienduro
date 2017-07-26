use elementtree::Element;

use std::fs::File;
use std::io;
use std::io::prelude::*;

use postgis::ewkb;

pub fn read_whole_file(filename: String) -> Result<String, io::Error> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    return Ok(contents);
}

pub fn parse_gpx(gpx_data: String) -> Result<ewkb::LineString, io::Error> {
    let mut points = Vec::new();

    let gpx = Element::from_reader(&mut gpx_data.as_bytes()).unwrap();
    let ns = "http://www.topografix.com/GPX/1/1";
    let trk = gpx.find((ns, "trk")).unwrap();
    let trkseg = trk.find((ns, "trkseg")).unwrap();
    for trkpt in trkseg.find_all((ns, "trkpt")) {
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        //let mut z: f64 = 0.0;
        match trkpt.get_attr("lat") {
            Some(val) => x = val.parse().unwrap(),
            None => (),
        }

        match trkpt.get_attr("lon") {
            Some(val) => y = val.parse().unwrap(),
            None => (),
        }

        //match trkpt.find((ns, "ele")) {
        //    Some(val) => z = val.text().parse().unwrap(),
        //    None => (),
        //}

        points.push(ewkb::Point{x, y, srid: None});
    }

    return Ok(ewkb::LineString{points, srid: Some(4326)});
}
