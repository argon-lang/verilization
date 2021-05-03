package enum_.versions;

import dev.argon.verilization.runtime.Converter;

final class Main_Conversions {
    private Main_Conversions() {}
    
    static final Converter<Main.V3, Main.V4> v3ToV4 = new Converter<Main.V3, Main.V4>() {
        @Override
        public Main.V4 convert(Main.V3 prev) {
            if(prev instanceof Main.V3.N) {
                return new Main.V4.N(((Main.V3.N)prev).n);
            }
            else if(prev instanceof Main.V3.M) {
                return new Main.V4.M(((Main.V3.M)prev).m);
            }
            else if(prev instanceof Main.V3.R) {
                return new Main.V4.R(Referenced.V4.fromV3.convert(((Main.V3.R)prev).r));
            }
            else {
                throw new IllegalArgumentException();
            }
        }
    };
    

}
