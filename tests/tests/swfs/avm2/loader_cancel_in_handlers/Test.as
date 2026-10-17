package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.HTTPStatusEvent;
    import flash.events.IOErrorEvent;
    import flash.events.ProgressEvent;
    import flash.net.URLLoader;
    import flash.net.URLLoaderDataFormat;
    import flash.net.URLRequest;
    import flash.utils.ByteArray;
    import flash.utils.getQualifiedClassName;

    public class Test extends Sprite {
        private static const CONTENT:Object = {
            swf: "a.swf", image: "image.png", missing: "missing.swf", unknown: "bad.txt",
            "swf bytes": "a.swf", "image bytes": "image.png"
        };
        private static const RELOAD:Object = {
            swf: "b.swf", image: "image.png", missing: "b.swf", unknown: "b.swf",
            "swf bytes": "b.swf", "image bytes": "image.png"
        };

        private var cases:Array = [];
        private var bytes:Object = {};
        private var current:Object;
        private var frames:int = 0;

        public function Test() {
            for each (var kind:String in ["swf", "image", "swf bytes", "image bytes"]) {
                for each (var event:String in [Event.INIT, Event.COMPLETE, ProgressEvent.PROGRESS]) {
                    addCases(kind, event);
                }
            }
            for each (kind in ["swf", "image", "missing", "unknown"]) {
                addCases(kind, HTTPStatusEvent.HTTP_STATUS);
            }
            addCases("missing", IOErrorEvent.IO_ERROR);
            addCases("unknown", IOErrorEvent.IO_ERROR);
            fetch("a.swf", function():void {
                fetch("b.swf", function():void {
                    fetch("image.png", function():void {
                        addEventListener(Event.ENTER_FRAME, onFrame);
                    });
                });
            });
        }

        private function addCases(kind:String, event:String):void {
            for each (var action:String in ["load", "loadBytes", "close"]) {
                cases.push({kind: kind, event: event, action: action});
            }
        }

        private function fetch(url:String, done:Function):void {
            var fetcher:URLLoader = new URLLoader();
            fetcher.dataFormat = URLLoaderDataFormat.BINARY;
            fetcher.addEventListener(Event.COMPLETE, function(e:Event):void {
                bytes[url] = fetcher.data;
                done();
            });
            fetcher.load(new URLRequest(url));
        }

        private function start(c:Object):void {
            var loader:Loader = new Loader();
            var info:Object = loader.contentLoaderInfo;
            var acted:Boolean = false;
            c.loader = loader;
            c.after = [];
            for each (var event:String in [Event.INIT, HTTPStatusEvent.HTTP_STATUS, Event.COMPLETE, IOErrorEvent.IO_ERROR, ProgressEvent.PROGRESS]) {
                info.addEventListener(event, function(e:Event):void {
                    if (acted && e.type != ProgressEvent.PROGRESS) {
                        c.after.push(e.type);
                    }
                    if (!acted && e.type == c.event) {
                        acted = true;
                        act(c);
                    }
                });
            }
            addChild(loader);
            if (c.kind.indexOf("bytes") != -1) {
                loader.loadBytes(bytes[CONTENT[c.kind]]);
            } else {
                loader.load(new URLRequest(CONTENT[c.kind]));
            }
        }

        private function act(c:Object):void {
            var reload:String = RELOAD[c.kind];
            if (c.action == "load") {
                c.loader.load(new URLRequest(reload));
            } else if (c.action == "loadBytes") {
                c.loader.loadBytes(bytes[reload]);
            } else {
                c.loader.close();
            }
        }

        private function finish(c:Object):void {
            var content:Object = c.loader.content;
            trace(c.action + " in " + c.event + " of " + c.kind + ": then [" + c.after.join(", ") + "], content "
                + (content == null ? "null" : getQualifiedClassName(content)));
            removeChild(c.loader);
        }

        private function onFrame(e:Event):void {
            if (current != null && ++frames < 8) {
                return;
            }
            if (current != null) {
                finish(current);
            }
            current = cases.shift();
            frames = 0;
            if (current == null) {
                trace("done");
                removeEventListener(Event.ENTER_FRAME, onFrame);
                return;
            }
            start(current);
        }
    }
}
