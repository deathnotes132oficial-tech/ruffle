package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Test extends Sprite {
        private var cases:Array = [pending, window, imageInit, nestedPending, nestedInit, nestedHttpStatus];
        private var loader:Loader;
        private var unloaded:Boolean;
        private var after:Array;
        private var frames:int = 0;

        public function Test() {
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function start(url:String):void {
            loader = new Loader();
            unloaded = false;
            after = [];
            for each (var type:String in [Event.INIT, Event.COMPLETE]) {
                loader.contentLoaderInfo.addEventListener(type, function(e:Event):void {
                    if (unloaded) {
                        after.push(e.type);
                    }
                });
            }
            for each (var nested:String in ["init", "httpStatus", "complete", "timer", "interval"]) {
                loader.contentLoaderInfo.sharedEvents.addEventListener("nested " + nested, function(e:Event):void {
                    if (unloaded && after.indexOf(e.type) < 0) {
                        after.push(e.type);
                    }
                });
            }
            addChild(loader);
            loader.load(new URLRequest(url));
        }

        private function stop(close:Boolean, gc:Boolean):void {
            unloaded = true;
            if (close) {
                loader.close();
            }
            loader.unloadAndStop(gc);
        }

        private function pending():void {
            trace("load, then unloadAndStop(true) in the same turn:");
            start("a.swf");
            stop(false, true);
        }

        private function window():void {
            trace("close() and unloadAndStop(true) once the content is in, before init:");
            start("a.swf");
            addEventListener(Event.ENTER_FRAME, function inWindow(e:Event):void {
                if (loader.content != null) {
                    removeEventListener(Event.ENTER_FRAME, inWindow);
                    trace("  content inserted, init not fired");
                    stop(true, true);
                }
            });
        }

        private function imageInit():void {
            trace("unloadAndStop() in an image's init:");
            start("image.png");
            loader.contentLoaderInfo.addEventListener(Event.INIT, function(e:Event):void {
                if (!unloaded) {
                    stop(false, false);
                }
            });
        }

        private function nestedPending():void {
            trace("unloadAndStop(true) in the init of a child whose own load is still pending:");
            start("child.swf");
            loader.contentLoaderInfo.addEventListener(Event.INIT, function(e:Event):void {
                if (!unloaded) {
                    stop(false, true);
                }
            });
        }

        private function nestedInit():void {
            trace("unloadAndStop(true) in the init of a load that the child started:");
            start("map.swf");
            stopOn("nested init");
        }

        private function nestedHttpStatus():void {
            trace("unloadAndStop(true) in the httpStatus of a load that the child started:");
            start("map.swf");
            stopOn("nested httpStatus");
        }

        private function stopOn(type:String):void {
            loader.contentLoaderInfo.sharedEvents.addEventListener(type, function(e:Event):void {
                if (!unloaded) {
                    stop(false, true);
                }
            });
        }

        private function onFrame(e:Event):void {
            if (frames++ % 10 != 0) {
                return;
            }
            if (loader != null) {
                trace("  events after unloading: [" + after.join(", ") + "], content " + loader.content);
                removeChild(loader);
            }
            if (cases.length == 0) {
                trace("done");
                removeEventListener(Event.ENTER_FRAME, onFrame);
                return;
            }
            cases.shift()();
        }
    }
}
