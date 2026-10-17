package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.net.URLRequest;

    public class Test extends Sprite {
        private var loaders:Array = [];
        private var loaded:int = 0;
        private var frames:int = -1;

        public function Test() {
            for each (var url:String in ["importer.swf", "exporter.swf"]) {
                var loader:Loader = new Loader();
                addChild(loader);
                loader.contentLoaderInfo.addEventListener(Event.COMPLETE, onComplete);
                loader.load(new URLRequest(url));
                loaders.push(loader);
            }
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function onComplete(e:Event):void {
            trace(e.target.url.split("/").pop() + " loaded");
            if (++loaded == loaders.length) {
                frames = 0;
            }
        }

        private function onFrame(e:Event):void {
            if (frames >= 0 && ++frames == 3) {
                for each (var loader:Loader in loaders) {
                    loader.unloadAndStop(true);
                }
                trace("importer and exporter unloaded");
            } else if (frames == 20) {
                trace("done");
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }
    }
}
