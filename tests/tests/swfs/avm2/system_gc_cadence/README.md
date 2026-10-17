Two collections requested three frames apart: `unloadAndStop(true)` of `a.swf`, then `unloadAndStop(true)` and `System.gc()` after `b.swf`. The player grants one requested collection per ten frames, so the second waits for the window to end. The Rust test `requested_gc_is_postponed_not_dropped` checks that `b.swf`'s library is gone by then.

Built with Apache Flex SDK 4.16.1 and playerglobal 32.0:

    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output test.swf Test.as
    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output a.swf A.as
    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output b.swf B.as
