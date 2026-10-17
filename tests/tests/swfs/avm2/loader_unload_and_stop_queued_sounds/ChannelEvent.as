package {
    import flash.events.Event;
    import flash.media.SoundChannel;

    public class ChannelEvent extends Event {
        public var channel:SoundChannel;

        public function ChannelEvent(channel:SoundChannel) {
            super("queued");
            this.channel = channel;
        }
    }
}
