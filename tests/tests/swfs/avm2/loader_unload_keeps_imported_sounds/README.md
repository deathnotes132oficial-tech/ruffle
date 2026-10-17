ImportAssets shares a sound in both directions. `importer.swf` takes the sound `beep` from `asset.swf`. `exporter.swf` defines and exports its own `beep` and imports from the otherwise empty `blank.swf`, which is given the exporter's exports. The Rust test `released_importer_keeps_imported_sound` checks that releasing their libraries leaves the sound playable from `asset.swf` and from `blank.swf`, both kept for the session, and `unregistered_sound_fails_to_start` that a sound that is no longer registered fails to start.

`asset.swf`, `importer.swf`, `exporter.swf` and `blank.swf` are written by `make_swfs.py` (`python3 make_swfs.py .`): mxmlc cannot emit ImportAssets. `test.swf` is built with Apache Flex SDK 4.16.1 and playerglobal 32.0:

    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output test.swf Test.as
