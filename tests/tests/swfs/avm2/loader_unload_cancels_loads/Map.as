package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.HTTPStatusEvent;
    import flash.events.TimerEvent;
    import flash.net.URLRequest;
    import flash.utils.Timer;
    import flash.utils.setInterval;

    public class Map extends Sprite {
        private var nested:Loader = new Loader();

        public function Map() {
            for each (var type:String in [Event.INIT, HTTPStatusEvent.HTTP_STATUS, Event.COMPLETE]) {
                nested.contentLoaderInfo.addEventListener(type, forward);
            }
            nested.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                var timer:Timer = new Timer(10);
                timer.addEventListener(TimerEvent.TIMER, forward);
                timer.start();
                setInterval(function():void {
                    forward(new Event("interval"));
                }, 10);
            });
            nested.load(new URLRequest("a.swf"));
        }

        private function forward(e:Event):void {
            loaderInfo.sharedEvents.dispatchEvent(new Event("nested " + e.type));
        }
    }
}
