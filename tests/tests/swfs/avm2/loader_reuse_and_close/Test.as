package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.IOErrorEvent;
    import flash.net.URLLoader;
    import flash.net.URLLoaderDataFormat;
    import flash.net.URLRequest;
    import flash.utils.ByteArray;
    import flash.utils.getQualifiedClassName;

    public class Test extends Sprite {
        private var loader:Loader = new Loader();
        private var bytesA:ByteArray;
        private var bytesB:ByteArray;
        private var step:int = 0;
        private var stepReady:Boolean = false;
        private var idleFrames:int = -1;

        public function Test() {
            addChild(loader);
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
            loader.contentLoaderInfo.addEventListener(IOErrorEvent.IO_ERROR, onError);
            addEventListener(Event.ENTER_FRAME, onFrame);
            fetch("a.swf", function(a:ByteArray):void {
                bytesA = a;
                fetch("b.swf", function(b:ByteArray):void {
                    bytesB = b;
                    stepReady = true;
                });
            });
        }

        private function fetch(url:String, done:Function):void {
            var fetcher:URLLoader = new URLLoader();
            fetcher.dataFormat = URLLoaderDataFormat.BINARY;
            fetcher.addEventListener(Event.COMPLETE, function(e:Event):void {
                done(fetcher.data);
            });
            fetcher.load(new URLRequest(url));
        }

        private function onComplete(e:Event):void {
            trace("complete: " + getQualifiedClassName(loader.content));
            stepReady = true;
        }

        private function onError(e:IOErrorEvent):void {
            trace("ioError: " + e.text);
            stepReady = true;
        }

        private function onFrame(e:Event):void {
            if (idleFrames >= 0 && ++idleFrames == 10) {
                idleFrames = -1;
                trace("content after close: " + loader.content);
                stepReady = true;
            }
            if (!stepReady) {
                return;
            }
            stepReady = false;
            step++;
            if (step == 1) {
                trace("load a, then load b");
                loader.load(new URLRequest("a.swf"));
                loader.load(new URLRequest("b.swf"));
            } else if (step == 2) {
                trace("load a, then close");
                loader.load(new URLRequest("a.swf"));
                loader.close();
                idleFrames = 0;
            } else if (step == 3) {
                trace("loadBytes a, then loadBytes b");
                loader.loadBytes(bytesA);
                loader.loadBytes(bytesB);
            } else if (step == 4) {
                trace("load a, then loadBytes b");
                loader.load(new URLRequest("a.swf"));
                loader.loadBytes(bytesB);
            } else if (step == 5) {
                trace("close with nothing loading");
                loader.close();
                trace("content: " + getQualifiedClassName(loader.content));
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
