use postgis::ewkb;

pub struct User {
    id: i64,
    name: String,
    email: String,
}

pub struct SourceRoute {
    id: i64,
    gpx: Option<String>,
}

pub struct Segment {
    id: i64,
    name: String,
    route_id: i64,
    source_id: i64,
    geom: Option<ewkb::LineString>,
    geom_expanded: Option<ewkb::Polygon>,
}

pub struct Event {
    id: i64,
    name: String,
}

pub struct Participation {
    id: i64,
    event_id: i64,
    user_id: i64,
    route_id: i64,
    source_id: i64,
    total_elapsed_seconds: Option<i64>,
    geom: Option<ewkb::LineString>,
}

pub struct ParticipationSegment {
    participation_id: i64,
    segment_id: i64,
    elapsed_seconds: Option<i64>,
    geom: Option<ewkb::LineString>,
}