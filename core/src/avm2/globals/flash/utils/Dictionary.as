package flash.utils {

    [Ruffle(InstanceAllocator)]
    public dynamic class Dictionary {
        prototype.toJSON = function(r:String):* {
            return "Dictionary";
        };
        prototype.setPropertyIsEnumerable("toJSON", false);

        public function Dictionary(weakKeys:Boolean = false) {
            if (weakKeys) {
                this.setWeakKeys();
            }
        }

        // skua #13: keys are held weakly (see dictionary_object.rs).
        private native function setWeakKeys():void;
    }
}
