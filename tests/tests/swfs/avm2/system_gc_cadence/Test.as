package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;
    import flash.system.System;

    public class Test extends Sprite {
        private var loader:Loader = new Loader();
        private var frame:int = 0;

        public function Test() {
            addChild(loader);
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function onFrame(e:Event):void {
            frame++;
            if (frame == 1) {
                loader.load(new URLRequest("a.swf"));
            } else if (frame == 6) {
                trace("a loaded: " + (loader.content != null));
                loader.unloadAndStop(true);
                loader.load(new URLRequest("b.swf"));
            } else if (frame == 9) {
                trace("b loaded: " + (loader.content != null));
                loader.unloadAndStop(true);
                System.gc();
            } else if (frame == 40) {
                trace("done");
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
