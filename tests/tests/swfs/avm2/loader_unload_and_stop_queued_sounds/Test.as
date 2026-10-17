package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.media.Sound;
    import flash.media.SoundChannel;
    import flash.net.URLRequest;

    public class Test extends Sprite {
        private var loader:Loader = new Loader();
        private var frame:int = -1;
        private var own:SoundChannel;
        private var child:SoundChannel;

        public function Test() {
            addChild(loader);
            loader.contentLoaderInfo.sharedEvents.addEventListener("queued", function(e:Event):void {
                child = Object(e).channel;
                own = new Sound(new URLRequest("noise.mp3")).play(0, 1000);
                loader.unloadAndStop(true);
                trace("unloaded while both sounds were loading");
                frame = 0;
            });
            addEventListener(Event.ENTER_FRAME, onFrame);
            loader.load(new URLRequest("child.swf"));
        }

        private function onFrame(e:Event):void {
            if (frame < 0) {
                return;
            }
            frame++;
            if (frame == 9) {
                trace("the child's queued sound played after unloading: " + (child.position > 0));
                trace("the parent's queued sound played after unloading: " + (own.position > 0));
                own.stop();
                own = null;
                child = null;
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
