package {
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.media.Sound;
    import flash.media.SoundChannel;

    public class Child extends Sprite {
        [Embed(source="noise.mp3")]
        private static const Noise:Class;

        public var channel:SoundChannel;

        public function Child() {
            channel = Sound(new Noise()).play(0, 1000);
            channel.addEventListener(Event.SOUND_COMPLETE, function(e:Event):void {
                trace("the child's sound completed");
            });
        }
    }
}
