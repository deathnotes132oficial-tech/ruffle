package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;
    import flash.utils.Dictionary;
    import flash.utils.setInterval;

    public class Test extends Sprite {
        private static const LIVE:int = 200000;
        private static const CHURN:int = 20000;

        private var loader:Loader = new Loader();
        private var frame:int = 0;
        private var live:Array = [];
        private var garbage:Array;
        private var unloaded:Dictionary = new Dictionary(true);
        private var release:Function;

        public function Test() {
            for (var i:int = 0; i < LIVE; i++) {
                live.push({index: i});
            }
            addChild(loader);
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function holdFromTimer(content:Object):void {
            release = function():void {
                content = null;
            };
            setInterval(function():void {
                content;
            }, 100000);
        }

        private function keepCollectorMarking():void {
            garbage = [];
            for (var i:int = 0; i < CHURN; i++) {
                garbage.push({index: i});
            }
        }

        private function onFrame(e:Event):void {
            frame++;
            keepCollectorMarking();
            if (frame == 1) {
                loader.load(new URLRequest("b.swf"));
            } else if (frame == 5) {
                holdFromTimer(loader.content);
            } else if (frame == 20) {
                trace("b loaded: " + (loader.content != null));
                unloaded[loader.content] = true;
                release();
                loader.unloadAndStop(true);
            } else if (frame == 21) {
                var left:int = 0;
                for (var key:Object in unloaded) {
                    left++;
                }
                trace("b collected by the next frame: " + (left == 0));
                trace("done");
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
