Adds 5,000 weak `ping` listeners to the stage, lets them be collected, and dispatches `ping` at frame 30. The Rust test `dead_weak_listeners_are_pruned_on_dispatch` checks that the dispatch prunes the dead entries, so that their allocations can be freed.

Built with Apache Flex SDK 4.16.1 and playerglobal 32.0:

    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output test.swf Test.as
