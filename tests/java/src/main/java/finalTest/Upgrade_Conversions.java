package finalTest;

import java.util.function.Function;

public class Upgrade_Conversions {
    private Upgrade_Conversions() {}

    static Upgrade.V4 v3ToV4(Upgrade.V3 prev) {
        return new Upgrade.V4(prev.n);
    }
    
}
