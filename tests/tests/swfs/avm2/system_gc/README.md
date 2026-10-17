`System.gc()` collects: a weak-key `Dictionary` loses its dead key by the next frame. Adobe documents that Flash Player's debugger and AIR collect, and its release builds ignore the call; the expected output follows that, and no Flash Player trace was available.

Built with Apache Flex SDK 4.16.1 and playerglobal 32.0:

    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output test.swf Test.as
