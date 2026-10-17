package {
    import flash.display.Sprite;
    import flash.events.Event;

    public class Test extends Sprite {
        private var frame:int = 0;

        public function Test() {
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function addWeakListeners():void {
            for (var i:int = 0; i < 5000; i++) {
                stage.addEventListener("ping", function(e:Event):void {
                }, false, 0, true);
            }
        }

        private function onFrame(e:Event):void {
            frame++;
            if (frame == 1) {
                addWeakListeners();
            } else if (frame == 30) {
                stage.dispatchEvent(new Event("ping"));
                trace("dispatched");
            } else if (frame == 40) {
                trace("done");
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
