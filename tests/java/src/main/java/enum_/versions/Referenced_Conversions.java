package enum_.versions;

import dev.argon.verilization.runtime.Converter;

final class Referenced_Conversions {
    private Referenced_Conversions() {}


    static final Converter<Referenced.V1, Referenced.V2> v1ToV2 = new Converter<Referenced.V1, Referenced.V2>() {
        @Override
        public Referenced.V2 convert(Referenced.V1 prev) {
            return switch(prev) {
                case Referenced.V1.X prev2 -> new Referenced.V2.X(prev2.x());
            };
        }
    };
    

}
