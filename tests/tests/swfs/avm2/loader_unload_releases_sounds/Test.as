package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Test extends Sprite {
        private var loader:Loader = new Loader();
        private var cycles:int = 0;
        private var framesLoaded:int = -1;

        public function Test() {
            addChild(loader);
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
            addEventListener(Event.ENTER_FRAME, onFrame);
            loader.load(new URLRequest("child/child.swf"));
        }

        private function onComplete(e:Event):void {
            framesLoaded = 0;
        }

        private function onFrame(e:Event):void {
            if (framesLoaded < 0 || ++framesLoaded < 3) {
                return;
            }
            framesLoaded = -1;
            loader.unloadAndStop(true);
            cycles++;
            trace("cycle " + cycles);
            if (cycles < 10) {
                loader.load(new URLRequest("child/child.swf"));
            } else {
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
