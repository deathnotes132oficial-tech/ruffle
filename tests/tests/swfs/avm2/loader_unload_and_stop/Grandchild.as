package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.TimerEvent;
    import flash.utils.Timer;

    public class Grandchild extends Sprite {
        public var stageFrames:int = 0;
        public var timerTicks:int = 0;

        public function Grandchild() {
            if (stage) {
                start();
            } else {
                addEventListener(Event.ADDED_TO_STAGE, onAdded);
            }
        }

        private function onAdded(e:Event):void {
            removeEventListener(Event.ADDED_TO_STAGE, onAdded);
            start();
        }

        private function start():void {
            stage.addEventListener(Event.ENTER_FRAME, onStageFrame);
            var timer:Timer = new Timer(10);
            timer.addEventListener(TimerEvent.TIMER, onTimer);
            timer.start();
        }

        private function onStageFrame(e:Event):void {
            stageFrames++;
        }

        private function onTimer(e:TimerEvent):void {
            timerTicks++;
        }
    }
}
