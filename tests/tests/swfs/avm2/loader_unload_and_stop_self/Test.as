package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Test extends Sprite {
        private var frames:int = 0;

        public function Test() {
            var loader:Loader = new Loader();
            addChild(loader);
            loader.load(new URLRequest("child.swf"));
            addEventListener(Event.ENTER_FRAME, function(e:Event):void {
                if (++frames == 15) {
                    trace("done, content " + loader.content);
                }
            });
        }
    }
}
