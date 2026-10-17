package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.events.UncaughtErrorEvent;
    import flash.net.URLRequest;
    import flash.utils.setTimeout;

    public class Test extends Sprite {
        private static var actions:Object = {};

        private var loader:Loader = new Loader();
        private var child:Object;
        private var frame:int = 0;

        public function Test() {
            listen(loaderInfo.uncaughtErrorEvents, "root loaderInfo");
            listen(loader.uncaughtErrorEvents, "loader");
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
            addChild(loader);
            loader.load(new URLRequest("child.swf"));
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
            if (action == "preventDefault") {
                e.preventDefault();
            } else if (action == "stopPropagation") {
                e.stopPropagation();
            } else if (action == "stopImmediatePropagation") {
                e.preventDefault();
                e.stopImmediatePropagation();
            }
        }

        private function onComplete(e:Event):void {
            child = loader.content;
            listen(loader.contentLoaderInfo.uncaughtErrorEvents, "child loaderInfo");
            trace("the Loader has its own uncaughtErrorEvents: "
                + (loader.uncaughtErrorEvents != loader.contentLoaderInfo.uncaughtErrorEvents));
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function onFrame(e:Event):void {
            frame++;
            if (frame == 1) {
                trace("boom1, thrown by a child listener, not cancelled:");
                child.throwInFrame("boom1");
            } else if (frame == 3) {
                trace("boom2, thrown by a child setTimeout, stopped at the target, cancelled by the loader:");
                actions["boom2 target child loaderInfo"] = "stopPropagation";
                actions["boom2 bubble loader"] = "preventDefault";
                child.throwInTimeout("boom2");
            } else if (frame == 6) {
                trace("boom3, thrown by the root, cancelled at its own loaderInfo:");
                actions["boom3 target root loaderInfo"] = "preventDefault";
                setTimeout(function():void {
                    throw new Error("boom3");
                }, 10);
            } else if (frame == 9) {
                trace("boom4, thrown by a child listener, cancelled and stopped by the loader's capture listener:");
                actions["boom4 capture loader"] = "stopImmediatePropagation";
                child.throwInFrame("boom4");
            } else if (frame == 12) {
                trace("done");
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
