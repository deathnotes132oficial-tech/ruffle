package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Child extends Sprite {
        private var nested:Loader = new Loader();

        public function Child() {
            nested.contentLoaderInfo.addEventListener(Event.INIT, function(e:Event):void {
                trace("  the nested load of the unloaded child finished");
            });
            nested.load(new URLRequest("a.swf"));
        }
    }
}
