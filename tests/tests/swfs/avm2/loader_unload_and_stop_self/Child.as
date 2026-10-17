package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.TimerEvent;
    import flash.utils.Timer;

    public class Child extends Sprite {
        private var frames:int = 0;
        private var timer:Timer = new Timer(10);
        private var stopped:Boolean = false;

        public function Child() {
            addEventListener(Event.ADDED_TO_STAGE, function(e:Event):void {
                stage.addEventListener(Event.ENTER_FRAME, onFrame);
            });
            timer.addEventListener(TimerEvent.TIMER, function(e:TimerEvent):void {
                if (stopped) {
                    trace("the child's Timer still ticks after unloading itself");
                    timer.stop();
                }
            });
            timer.start();
        }

        private function onFrame(e:Event):void {
            if (stopped) {
                trace("the child's stage listener still runs after unloading itself");
                stage.removeEventListener(Event.ENTER_FRAME, onFrame);
            } else if (++frames == 3) {
                stopped = true;
                loaderInfo.loader.unloadAndStop(true);
                trace("the child unloaded itself, Timer running: " + timer.running);
            }
        }
    }
}
