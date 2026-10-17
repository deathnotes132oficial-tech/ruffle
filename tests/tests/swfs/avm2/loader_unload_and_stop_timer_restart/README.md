A Timer that the parent created and the child started stops with the child on `Loader.unloadAndStop`: `running` turns false, and the parent can start it again. The expected output follows Adobe's documentation of `Timer` and `unloadAndStop`; no Flash Player trace was available.

Built with Apache Flex SDK 4.16.1 and playerglobal 32.0:

    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output test.swf Test.as
    mxmlc -target-player 32.0 -swf-version 15 -default-size 550 400 -omit-trace-statements=false -debug=false -static-link-runtime-shared-libraries=true -source-path . -output child.swf Child.as
