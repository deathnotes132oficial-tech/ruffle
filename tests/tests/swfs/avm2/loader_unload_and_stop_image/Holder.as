package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Holder extends Sprite {
        public var image:Loader = new Loader();
        public var ready:Boolean = false;

        public function Holder() {
            addChild(image);
            image.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
            image.load(new URLRequest("image.png"));
        }

        private function onComplete(e:Event):void {
            ready = true;
        }
    }
}
