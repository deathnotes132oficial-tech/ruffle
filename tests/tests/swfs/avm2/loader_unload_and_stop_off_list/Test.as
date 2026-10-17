package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Test extends Sprite {
        private var loader:Loader = new Loader();
        private var grandchild:Object;
        private var frames:int = -1;
        private var atUnload:Array;

        public function Test() {
            addChild(loader);
            stage.addEventListener(Event.ENTER_FRAME, onFrame);
            loader.load(new URLRequest("child.swf"));
        }

        private function counts():Array {
            return [grandchild.stageFrames, grandchild.ownFrames, grandchild.timerTicks];
        }

        private function onFrame(e:Event):void {
            if (grandchild == null) {
                if (loader.content != null && Object(loader.content).ready) {
                    grandchild = Object(loader.content).grandchild;
                    trace("grandchild on the display list: " + (grandchild.stage != null));
                    frames = 0;
                }
                return;
            }

            frames++;
            if (frames == 5) {
                var c:Array = counts();
                trace("grandchild runs before unload: stage enterFrame " + (c[0] > 0) + ", own enterFrame " + (c[1] > 0) + ", Timer " + (c[2] > 0));
                loader.unloadAndStop(true);
            } else if (frames == 6) {
                atUnload = counts();
            } else if (frames == 26) {
                var after:Array = counts();
                trace("grandchild stage enterFrame stopped: " + (after[0] == atUnload[0]));
                trace("grandchild own enterFrame stopped: " + (after[1] == atUnload[1]));
                trace("grandchild Timer stopped: " + (after[2] == atUnload[2]));
                trace("done");
                stage.removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
