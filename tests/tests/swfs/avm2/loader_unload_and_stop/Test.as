package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.TimerEvent;
    import flash.net.URLRequest;
    import flash.system.fscommand;
    import flash.utils.Timer;

    public class Test extends Sprite {
        private var loader:Loader = new Loader();
        private var child:Object;
        private var ownTimer:Timer = new Timer(10);
        private var ownTicks:int = 0;
        private var parentFramesOnChild:int = 0;
        private var framesSinceReady:int = -1;
        private var atUnload:Object;
        private var ownTicksAtUnload:int;

        public function Test() {
            addChild(loader);
            stage.addEventListener(Event.ENTER_FRAME, onFrame);
            ownTimer.addEventListener(TimerEvent.TIMER, onOwnTimer);
            ownTimer.start();
            loader.load(new URLRequest("child.swf"));
        }

        private function onOwnTimer(e:TimerEvent):void {
            ownTicks++;
        }

        private function onParentFrameOnChild(e:Event):void {
            parentFramesOnChild++;
        }

        private function counts():Object {
            return {
                childStage: child.stageFrames,
                childOwn: child.ownFrames,
                childTimer: child.timerTicks,
                childInterval: child.intervalTicks,
                grandchildStage: child.grandchild.stageFrames,
                grandchildTimer: child.grandchild.timerTicks,
                parentOnChild: parentFramesOnChild
            };
        }

        private function onFrame(e:Event):void {
            if (child == null) {
                if (loader.content == null || !Object(loader.content).ready) {
                    return;
                }
                child = loader.content;
                child.addEventListener(Event.ENTER_FRAME, onParentFrameOnChild);
                framesSinceReady = 0;
                return;
            }

            framesSinceReady++;
            if (framesSinceReady == 5) {
                var c:Object = counts();
                var running:Boolean = true;
                for (var key:String in c) {
                    if (c[key] <= 0) {
                        trace("not running before unload: " + key);
                        running = false;
                    }
                }
                trace("running before unload: " + running);
                trace("stage keyDown listener before unload: " + stage.hasEventListener("keyDown"));
                trace("child activate listener before unload: " + child.hasEventListener("activate"));
                child.armTimeout();
                loader.unloadAndStop(true);
            } else if (framesSinceReady == 6) {
                atUnload = counts();
                ownTicksAtUnload = ownTicks;
            } else if (framesSinceReady == 26) {
                var after:Object = counts();
                trace("child stage enterFrame stopped: " + (after.childStage == atUnload.childStage));
                trace("child enterFrame stopped: " + (after.childOwn == atUnload.childOwn));
                trace("child Timer stopped: " + (after.childTimer == atUnload.childTimer));
                trace("child setInterval stopped: " + (after.childInterval == atUnload.childInterval));
                trace("child setTimeout cancelled: " + !child.timeoutFired);
                trace("grandchild stage enterFrame stopped: " + (after.grandchildStage == atUnload.grandchildStage));
                trace("grandchild Timer stopped: " + (after.grandchildTimer == atUnload.grandchildTimer));
                trace("parent enterFrame listener on child stopped: " + (after.parentOnChild == atUnload.parentOnChild));
                trace("stage keyDown listener after unload: " + stage.hasEventListener("keyDown"));
                trace("child activate listener after unload: " + child.hasEventListener("activate"));
                trace("parent Timer still runs: " + (ownTicks > ownTicksAtUnload));
                ownTimer.stop();
                stage.removeEventListener(Event.ENTER_FRAME, onFrame);
                trace("done");
                fscommand("quit");
            }
        }
    }
}
