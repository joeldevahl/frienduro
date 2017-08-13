use postgis::ewkb;

pub struct User {
    id: i32,
    name: String,
    email: String,
}

pub struct Segment {
    id: i32,
    name: String,
    geom: ewkb::LineString,
}
