package {
    import flash.display.Sprite;
    import flash.display.Stage;
    import flash.events.Event;
    import flash.events.TimerEvent;
    import flash.utils.Timer;

    public class Grandchild extends Sprite {
        public var stageFrames:int = 0;
        public var ownFrames:int = 0;
        public var timerTicks:int = 0;

        public function start(stage:Stage):void {
            stage.addEventListener(Event.ENTER_FRAME, onStageFrame);
            addEventListener(Event.ENTER_FRAME, onOwnFrame);
            var timer:Timer = new Timer(10);
            timer.addEventListener(TimerEvent.TIMER, onTimer);
            timer.start();
        }

        private function onStageFrame(e:Event):void {
            stageFrames++;
        }

        private function onOwnFrame(e:Event):void {
            ownFrames++;
        }

        private function onTimer(e:TimerEvent):void {
            timerTicks++;
        }
    }
}
