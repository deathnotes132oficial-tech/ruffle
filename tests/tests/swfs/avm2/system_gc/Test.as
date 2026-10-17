package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.system.System;
    import flash.utils.Dictionary;

    public class Test extends Sprite {
        private var frame:int = 0;
        private var held:Object = {name: "held"};
        private var cache:Dictionary = new Dictionary(true);

        public function Test() {
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function onFrame(e:Event):void {
            frame++;
            if (frame == 1) {
                cache[held] = "held";
                cache[{name: "dropped"}] = "dropped";
                trace("keys before System.gc(): " + keys());
            } else if (frame == 2) {
                try {
                    System.gc();
                    System.gc();
                } catch (e:Error) {
                    trace("System.gc() threw " + e);
                }
            } else if (frame == 3) {
                trace("keys after System.gc(): " + keys());
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }

        private function keys():String {
            var names:Array = [];
            for (var key:Object in cache) {
                names.push(cache[key]);
            }
            names.sort();
            return names.join(", ");
        }
    }
}
