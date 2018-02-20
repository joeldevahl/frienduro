@echo off
target\debug\create_db
target\debug\create_user --name "Joel de Vahl" --email "joel@devahl.com"
target\debug\create_user --name "Marika de Vahl" --email "marika@devahl.com"
target\debug\create_segment --gpx "gpx\Harnon_Runt_2017_joel.gpx" --name "Harnon Runt" --splits 9 --pad 5
target\debug\create_event --name "Test Race" 1 2 3 4 5 6 7 8 9 10
target\debug\create_participation --uid 1 --eid 1 --gpx "gpx\Harnon_Runt_2017_joel.gpx"
target\debug\update_participation --pid 1
target\debug\update_participation --pid 1
target\debug\create_participation --uid 2 --eid 1 --gpx "gpx\Harnon_Runt_2017_marika.gpx"
target\debug\update_participation --pid 2
target\debug\update_event --eid 1
