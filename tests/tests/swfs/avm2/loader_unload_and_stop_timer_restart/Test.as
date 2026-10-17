package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.TimerEvent;
    import flash.net.URLRequest;
    import flash.utils.Timer;

    public class Test extends Sprite {
        private var loader:Loader = new Loader();
        private var timer:Timer = new Timer(10);
        private var ticks:int = 0;
        private var frames:int = -1;
        private var ticksAtRestart:int;

        public function Test() {
            addChild(loader);
            timer.addEventListener(TimerEvent.TIMER, onTick);
            stage.addEventListener(Event.ENTER_FRAME, onFrame);
            loader.load(new URLRequest("child.swf"));
        }

        private function onTick(e:TimerEvent):void {
            ticks++;
        }

        private function onFrame(e:Event):void {
            if (frames == -1) {
                if (loader.content != null) {
                    Object(loader.content).startTimer(timer);
                    trace("started by the child, running: " + timer.running);
                    frames = 0;
                }
                return;
            }

            frames++;
            if (frames == 5) {
                trace("ticks before unload: " + (ticks > 0));
                loader.unloadAndStop(true);
                trace("running after unloadAndStop: " + timer.running);
                ticksAtRestart = ticks;
                timer.start();
                trace("restarted by the parent, running: " + timer.running);
            } else if (frames == 15) {
                trace("ticks after the restart: " + (ticks > ticksAtRestart));
                timer.stop();
                trace("done");
                stage.removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
