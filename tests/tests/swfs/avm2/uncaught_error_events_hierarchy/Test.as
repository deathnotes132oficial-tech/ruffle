package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.events.UncaughtErrorEvent;
    import flash.net.URLRequest;

    public class Test extends Sprite {
        private static var actions:Object = {};

        private var loader:Loader = new Loader();
        private var offList:Loader = new Loader();
        private var loaded:int = 0;
        private var frame:int = 0;

        public function Test() {
            listen(loaderInfo.uncaughtErrorEvents, "root loaderInfo");
            listen(loader.uncaughtErrorEvents, "loader");
            listen(offList.uncaughtErrorEvents, "off-list loader");
            listen(loader.contentLoaderInfo.uncaughtErrorEvents, "child loaderInfo");
            listen(offList.contentLoaderInfo.uncaughtErrorEvents, "off-list child loaderInfo");
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
            offList.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
            addChild(loader);
            loader.load(new URLRequest("child.swf"));
            offList.load(new URLRequest("child.swf"));
        }

        private static function listen(target:EventDispatcher, name:String):void {
            target.addEventListener(UncaughtErrorEvent.UNCAUGHT_ERROR, function(e:UncaughtErrorEvent):void {
                report("capture " + name, e);
            }, true);
            target.addEventListener(UncaughtErrorEvent.UNCAUGHT_ERROR, function(e:UncaughtErrorEvent):void {
                report((e.eventPhase == 2 ? "target " : "bubble ") + name, e);
            });
        }

        private static function report(where:String, e:UncaughtErrorEvent):void {
            var message:String = e.error.message;
            trace("  " + where + ": " + message);
            var action:String = actions[message + " " + where];
            if (action == "stopPropagation") {
                e.preventDefault();
                e.stopPropagation();
            } else if (action == "preventDefault") {
                e.preventDefault();
            }
        }

        private function onComplete(e:Event):void {
            if (++loaded == 2) {
                addEventListener(Event.ENTER_FRAME, onFrame);
            }
        }

        private function onFrame(e:Event):void {
            frame++;
            if (frame == 1) {
                trace("boom1, cancelled and stopped by the loader's capture listener:");
                actions["boom1 capture loader"] = "stopPropagation";
                Object(loader.content).throwInFrame("boom1");
            } else if (frame == 3) {
                trace("boom2, thrown by the child of a Loader off the display list, cancelled by the root:");
                actions["boom2 bubble root loaderInfo"] = "preventDefault";
                Object(offList.content).throwInFrame("boom2");
            } else if (frame == 5) {
                trace("done");
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
