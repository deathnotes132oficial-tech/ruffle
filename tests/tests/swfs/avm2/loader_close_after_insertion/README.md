`Loader.close` once the content is in the Loader but before its `init` event (in the content's `added` event, and in a later `enterFrame`) leaves the load to finish: `init` and `complete` still fire. There is nothing left to cancel once the bytes are in. The expected output follows Adobe's documentation of `Loader.close` ("Cancels a load() method operation that is currently in progress"); no Flash Player trace was available.

Built with Apache Flex SDK 4.16.1 and playerglobal 32.0:

    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output test.swf Test.as
    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output a.swf A.as
