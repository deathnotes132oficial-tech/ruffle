package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;
    import flash.utils.getQualifiedClassName;

    public class Test extends Sprite {
        private var loader:Loader;
        private var events:Array;
        private var closed:Boolean;
        private var frames:int;
        private var step:int = 0;

        public function Test() {
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function start():void {
            loader = new Loader();
            events = [];
            closed = false;
            frames = 0;
            for each (var type:String in [Event.INIT, Event.COMPLETE]) {
                loader.contentLoaderInfo.addEventListener(type, function(e:Event):void {
                    events.push(e.type);
                });
            }
            addChild(loader);
            loader.load(new URLRequest("a.swf"));
        }

        private function closeOnce(where:String):void {
            if (!closed && loader.content != null && events.length == 0) {
                closed = true;
                loader.close();
                trace("closed " + where + ", before init");
            }
        }

        private function report():void {
            trace("then [" + events.join(", ") + "], content " + getQualifiedClassName(loader.content));
            removeChild(loader);
        }

        private function onAdded(e:Event):void {
            closeOnce("in the content's added event");
        }

        private function onFrame(e:Event):void {
            if (step == 0) {
                start();
                loader.addEventListener(Event.ADDED, onAdded);
                step = 1;
            } else if (step == 1 && ++frames == 10) {
                report();
                start();
                step = 2;
            } else if (step == 2) {
                closeOnce("in a later enterFrame");
                if (++frames == 10) {
                    report();
                    trace("done");
                    step = 3;
                }
            }
        }
    }
}
