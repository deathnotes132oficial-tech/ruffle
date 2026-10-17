package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.KeyboardEvent;
    import flash.events.TimerEvent;
    import flash.net.URLRequest;
    import flash.utils.Timer;
    import flash.utils.setInterval;
    import flash.utils.setTimeout;

    public class Child extends Sprite {
        public var stageFrames:int = 0;
        public var ownFrames:int = 0;
        public var timerTicks:int = 0;
        public var intervalTicks:int = 0;
        public var timeoutFired:Boolean = false;
        public var grandchild:Object;
        public var ready:Boolean = false;
        private var nested:Loader = new Loader();

        public function Child() {
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
            stage.addEventListener(KeyboardEvent.KEY_DOWN, onKeyDown);
            addEventListener(Event.ENTER_FRAME, onOwnFrame);
            addEventListener(Event.ACTIVATE, onActivate);
            var timer:Timer = new Timer(10);
            timer.addEventListener(TimerEvent.TIMER, onTimer);
            timer.start();
            setInterval(onInterval, 10);
            nested.contentLoaderInfo.addEventListener(Event.COMPLETE, onNestedComplete);
            addChild(nested);
            nested.load(new URLRequest("grandchild.swf"));
        }

        public function armTimeout():void {
            setTimeout(onTimeout, 200);
        }

        private function onNestedComplete(e:Event):void {
            grandchild = nested.content;
            ready = true;
        }

        private function onStageFrame(e:Event):void {
            stageFrames++;
        }

        private function onKeyDown(e:KeyboardEvent):void {
        }

        private function onOwnFrame(e:Event):void {
            ownFrames++;
        }

        private function onActivate(e:Event):void {
        }

        private function onTimer(e:TimerEvent):void {
            timerTicks++;
        }

        private function onInterval():void {
            intervalTicks++;
        }

        private function onTimeout():void {
            timeoutFired = true;
        }
    }
}
