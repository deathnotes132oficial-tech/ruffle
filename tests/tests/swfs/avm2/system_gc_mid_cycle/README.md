`unloadAndStop(true)` while the collector is in the middle of a cycle that already marked the content: a timer closure holds the content, so the cycle reaches it early, and a large live set keeps the cycle running past the unload. The collection that grants the request finishes that cycle and runs a new one, so the content (a key of a weak-key `Dictionary`) is gone by the next frame. Before that, the content survived the requested collection.

Built with Apache Flex SDK 4.16.1 and playerglobal 32.0:

    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output test.swf Test.as
    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output b.swf B.as
