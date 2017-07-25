use elementtree::Element;

use std::fs::File;
use std::io;
use std::io::prelude::*;

use postgis::ewkb;

pub fn parse_gpx(gpx_file: String) -> Result<ewkb::LineString, io::Error> {
    let mut points = Vec::new();

    let mut file = File::open(gpx_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let gpx = Element::from_reader(&mut contents.as_bytes()).unwrap();
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

    return Ok(ewkb::LineString{points, srid: Some(43256)});
}
