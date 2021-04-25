package finalTest;

import dev.argon.verilization.java_runtime.Converter;

import java.util.function.Function;

public class Upgrade_Conversions {
    private Upgrade_Conversions() {}

    static final Converter<Upgrade.V3, Upgrade.V4> v3ToV4 = new Converter<Upgrade.V3, Upgrade.V4>() {
        @Override
        public Upgrade.V4 convert(Upgrade.V3 prev) {
            return new Upgrade.V4(prev.n);
        }
    };
    
}
