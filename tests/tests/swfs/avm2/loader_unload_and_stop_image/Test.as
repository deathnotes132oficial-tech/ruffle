package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.TimerEvent;
    import flash.net.URLRequest;
    import flash.utils.Timer;

    public class Test extends Sprite {
        private var imageLoader:Loader = new Loader();
        private var holderLoader:Loader = new Loader();
        private var timer:Timer = new Timer(10);
        private var stageFrames:int = 0;
        private var ownFrames:int = 0;
        private var ticks:int = 0;
        private var before:Array;

        public function Test() {
            addChild(imageLoader);
            addChild(holderLoader);
            stage.addEventListener(Event.ENTER_FRAME, onStageFrame);
            addEventListener(Event.ENTER_FRAME, onOwnFrame);
            timer.addEventListener(TimerEvent.TIMER, onTick);
            timer.start();
            imageLoader.contentLoaderInfo.addEventListener(Event.COMPLETE, onImage);
            imageLoader.load(new URLRequest("image.png"));
        }

        private function onTick(e:TimerEvent):void {
            ticks++;
        }

        private function onStageFrame(e:Event):void {
            stageFrames++;
        }

        private function onOwnFrame(e:Event):void {
            ownFrames++;
        }

        private function counts():Array {
            return [stageFrames, ownFrames, ticks];
        }

        private function afterProbeLoads(then:Function, loads:int = 8):void {
            var probe:Loader = new Loader();
            probe.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                if (loads > 1) {
                    afterProbeLoads(then, loads - 1);
                } else {
                    then();
                }
            });
            probe.load(new URLRequest("tick.swf"));
        }

        private function unloadAndReport(loader:Loader, what:String, then:Function):void {
            loader.unloadAndStop(true);
            before = counts();
            afterProbeLoads(function():void {
                var now:Array = counts();
                trace("after unloadAndStop of " + what + ": parent stage listener kept " + stage.hasEventListener("enterFrame")
                    + ", own listener kept " + hasEventListener("enterFrame")
                    + ", stage enterFrame fires " + (now[0] > before[0])
                    + ", own enterFrame fires " + (now[1] > before[1])
                    + ", Timer ticks " + (now[2] > before[2]) + ", running " + timer.running);
                then();
            });
        }

        private function onImage(e:Event):void {
            trace("image loaded: " + imageLoader.content);
            afterProbeLoads(function():void {
                unloadAndReport(imageLoader, "an image", loadHolder);
            }, 1);
        }

        private function loadHolder():void {
            holderLoader.contentLoaderInfo.addEventListener(Event.COMPLETE, onHolder);
            holderLoader.load(new URLRequest("holder.swf"));
        }

        private function onHolder(e:Event):void {
            afterProbeLoads(function():void {
                trace("holder loaded its image: " + Object(holderLoader.content).image.content);
                unloadAndReport(holderLoader, "a SWF holding an image Loader", function():void {
                    trace("done");
                });
            }, 2);
        }
    }
}
