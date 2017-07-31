CREATE EXTENSION postgis;

CREATE TABLE users (
	id SERIAL PRIMARY KEY,
	name VARCHAR NOT NULL,
	email VARCHAR NOT NULL
);

CREATE TABLE source_routes (
	id SERIAL PRIMARY KEY,
	gpx XML
);

CREATE TABLE segments (
	id SERIAL PRIMARY KEY,
	name VARCHAR NOT NULL,
	route GEOMETRY(LINESTRING,4326) NOT NULL,
	source_id INTEGER REFERENCES source_routes(id)
);

CREATE TABLE events (
	id SERIAL PRIMARY KEY,
	name VARCHAR NOT NULL
);

CREATE TABLE event_segments (
	event_id INTEGER REFERENCES events(id) ON UPDATE CASCADE ON DELETE CASCADE,
	segment_id INTEGER REFERENCES segments(id) ON UPDATE CASCADE,
	CONSTRAINT event_segments_pkey PRIMARY KEY (event_id, segment_id)
);

CREATE TABLE participations (
	id SERIAL PRIMARY KEY,
	event_id INTEGER REFERENCES events(id),
	user_id INTEGER REFERENCES users(id),
	route GEOMETRY(LINESTRING,4326) NOT NULL,
	source_id INTEGER REFERENCES source_routes(id),
	total_elapsed INTERVAL DEFAULT NULL
);
