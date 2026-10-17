A `Sound.play` call on a sound that is still loading waits until the sound loads. `Loader.unloadAndStop` drops the calls that the unloaded content's code made, so its looping sound never starts, while the parent's queued sound still starts. The Rust test `unloading_drops_queued_sounds` runs this movie and checks that the child's library is released. The expected output follows Adobe's documentation of `unloadAndStop`; no Flash Player trace was available.

Built with Apache Flex SDK 4.16.1 and playerglobal 32.0:

    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output test.swf Test.as
    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output child.swf Child.as

`noise.mp3` comes from `soundchannel_stop`.
