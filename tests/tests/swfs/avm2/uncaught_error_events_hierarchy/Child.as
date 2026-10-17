package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.utils.setTimeout;

    public class Child extends Sprite {
        private var pending:String;

        public function Child() {
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        public function throwInFrame(message:String):void {
            pending = message;
        }

        public function throwInTimeout(message:String):void {
            setTimeout(function():void {
                throw new Error(message);
            }, 10);
        }

        private function onFrame(e:Event):void {
            if (pending != null) {
                var message:String = pending;
                pending = null;
                throw new Error(message);
            }
        }
    }
}
