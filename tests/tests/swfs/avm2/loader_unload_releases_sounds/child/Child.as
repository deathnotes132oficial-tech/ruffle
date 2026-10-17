package {
    import flash.display.Sprite;
    import flash.media.Sound;

    public class Child extends Sprite {
        [Embed(source="beep.mp3")]
        private static const Beep:Class;

        public function Child() {
            Sound(new Beep()).play();
        }
    }
}
