target\debug\create_db
target\debug\create_user --name "Joel de Vahl" --email "joel@devahl.com"
target\debug\create_segment --gpx "gpx\Sundsvall_Classics_2017.gpx" --name "Sundsvall Classics" --splits 9 --pad 1
target\debug\create_event --name "Test Race" 1 2 3 4 5 6 7 8 9 10
target\debug\create_participation --uid 1 --eid 1 --gpx "gpx\Sundsvall_Classics_2017.gpx"
target\debug\update_participation --pid 1
