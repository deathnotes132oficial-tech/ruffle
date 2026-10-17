Unloading a Loader cancels its load, whatever stage the load has reached, and `unloadAndStop` also cancels the loads that the unloaded content started. A load that the content started gets no more events once its `init` or `httpStatus` handler unloads the content, so its `complete` handler never runs and cannot start a Timer or interval that outlives the unload. The Rust test `unloading_cancels_loads` runs this movie and checks that no load stays registered and that none of the unloaded movies stays resident. The expected output follows Adobe's documentation of `Loader.load`, `Loader.unload` and `Loader.unloadAndStop`; no Flash Player trace was available.

Built with Apache Flex SDK 4.16.1 and playerglobal 32.0:

    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output test.swf Test.as
    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output a.swf A.as
    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output child.swf Child.as
    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output map.swf Map.as
