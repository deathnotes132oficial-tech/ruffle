package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Child extends Sprite {
        public var grandchild:Object;
        public var ready:Boolean = false;
        private var nested:Loader = new Loader();

        public function Child() {
            nested.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
            nested.load(new URLRequest("grandchild.swf"));
        }

        private function onComplete(e:Event):void {
            grandchild = nested.content;
            grandchild.start(stage);
            ready = true;
        }
    }
}
