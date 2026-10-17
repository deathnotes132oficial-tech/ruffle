package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.UncaughtErrorEvent;
    import flash.net.URLRequest;
    import flash.system.System;

    public class Test extends Sprite {
        private var holder:Loader = new Loader();
        private var nested:Loader;
        private var caught:int = 0;
        private var frames:int = 0;

        public function Test() {
            addChild(holder);
            loaderInfo.uncaughtErrorEvents.addEventListener(UncaughtErrorEvent.UNCAUGHT_ERROR, function(e:UncaughtErrorEvent):void {
                caught++;
                trace("root caught");
                e.preventDefault();
            });
            holder.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                nested = Object(holder.content).nested;
                addChild(nested);
                nested.load(new URLRequest("child.swf"));
                holder.unload();
                trace("holder unloaded, nested still in root");
            });
            addEventListener(Event.ENTER_FRAME, function(e:Event):void {
                System.gc();
                if (++frames == 30) {
                    trace("root errors caught " + caught);
                }
            });
            holder.load(new URLRequest("holder.swf"));
        }
    }
}
