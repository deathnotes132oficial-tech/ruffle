package {
    import flash.display.Loader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.utils.Dictionary;

    public class Test extends Sprite {
        private var frame:int = 0;
        private var keptPanel:Panel;
        private var strongHandler:Function;
        private var lonely:EventDispatcher = new EventDispatcher();
        private var panels:Dictionary = new Dictionary(true);

        public function Test() {
            addEventListener(Event.ENTER_FRAME, onFrame);
        }

        private function onFrame(e:Event):void {
            frame++;
            if (frame == 1) {
                var weakPanel:Panel = new Panel("weak");
                var strongPanel:Panel = new Panel("strong");
                var lonelyPanel:Panel = new Panel("lonely");
                keptPanel = new Panel("kept");
                weakPanel.listen(stage, true);
                strongPanel.listen(stage, false);
                keptPanel.listen(stage, true);
                keptPanel.listen(stage, false);
                stage.addEventListener("ping", function(e:Event):void {
                    trace("  anonymous listener");
                }, false, 0, true);
                lonely.addEventListener("ping", lonelyPanel.onPing, false, 0, true);
                strongHandler = strongPanel.onPing;
                panels[weakPanel] = "weak";
                panels[strongPanel] = "strong";
                panels[keptPanel] = "kept";
                panels[lonelyPanel] = "lonely";

                trace("before collection:");
                stage.dispatchEvent(new Event("ping"));
                trace("lonely has a ping listener: " + lonely.hasEventListener("ping"));
            } else if (frame == 2) {
                requestFullCollection();
            } else if (frame == 3) {
                trace("after collection:");
                stage.dispatchEvent(new Event("ping"));
                trace("lonely has a ping listener: " + lonely.hasEventListener("ping"));
                trace("panels alive: " + alive());

                keptPanel.unlisten(stage);
                stage.removeEventListener("ping", strongHandler);
                trace("stage has a ping listener: " + stage.hasEventListener("ping"));
                removeEventListener(Event.ENTER_FRAME, onFrame);
            }
        }

        private function requestFullCollection():void {
            new Loader().unloadAndStop(true);
        }

        private function alive():String {
            var names:Array = [];
            for (var key:Object in panels) {
                names.push(panels[key]);
            }
            names.sort();
            return names.join(", ");
        }
    }
}

import flash.events.Event;
import flash.events.IEventDispatcher;

class Panel {
    private var name:String;

    public function Panel(name:String) {
        this.name = name;
    }

    public function listen(target:IEventDispatcher, weak:Boolean):void {
        target.addEventListener("ping", onPing, false, 0, weak);
    }

    public function unlisten(target:IEventDispatcher):void {
        target.removeEventListener("ping", onPing);
    }

    public function onPing(e:Event):void {
        trace("  " + name + " listener");
    }
}
