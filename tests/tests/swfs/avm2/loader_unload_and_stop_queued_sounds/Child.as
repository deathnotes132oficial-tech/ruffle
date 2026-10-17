package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.media.Sound;
    import flash.media.SoundChannel;
    import flash.net.URLRequest;

    public class Child extends Sprite {
        public function Child() {
            var sound:Sound = new Sound(new URLRequest("noise.mp3"));
            var channel:SoundChannel = sound.play(0, 1000);
            channel.addEventListener(Event.SOUND_COMPLETE, function(e:Event):void {
                trace("the child's sound completed");
            });
            loaderInfo.sharedEvents.dispatchEvent(new ChannelEvent(channel));
        }
    }
}
