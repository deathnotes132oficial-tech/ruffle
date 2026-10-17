package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.media.Sound;
    import flash.media.SoundChannel;
    import flash.net.URLRequest;

    public class Test extends Sprite {
        [Embed(source="noise.mp3")]
        private static const Noise:Class;

        private var loader:Loader = new Loader();
        private var frame:int = -1;
        private var own:SoundChannel;
        private var child:SoundChannel;
        private var ownPosition:Number;
        private var childPosition:Number;

        public function Test() {
            addChild(loader);
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                frame = 0;
                own = Sound(new Noise()).play(0, 1000);
                child = Object(loader.content).channel;
            });
            addEventListener(Event.ENTER_FRAME, onFrame);
            loader.load(new URLRequest("child.swf"));
        }

        private function onFrame(e:Event):void {
            if (frame < 0) {
                return;
            }
            frame++;
            if (frame == 3) {
                ownPosition = own.position;
                childPosition = child.position;
                loader.unloadAndStop(true);
                trace("unloaded");
            } else if (frame == 9) {
                trace("the child's sound advanced after unloading: " + (child.position > childPosition));
                trace("the parent's sound advanced after unloading: " + (own.position > ownPosition));
                own.stop();
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
