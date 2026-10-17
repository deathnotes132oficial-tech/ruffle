package {
    import flash.display.Sprite;
    import flash.events.Event;

    public class Child extends Sprite {
        private var frames:int = 0;

        public function Child() {
            addEventListener(Event.ENTER_FRAME, function(e:Event):void {
                if (++frames == 22) {
                    trace("throwing child error");
                    throw new Error("from surviving child");
                }
            });
        }
    }
}
