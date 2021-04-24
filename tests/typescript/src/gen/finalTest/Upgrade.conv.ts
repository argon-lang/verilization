import {V3, V4} from "./Upgrade.js";
import {Converter} from "@verilization/runtime";

export const v3_to_v4: Converter<V3, V4> = {
    convert(prev: V3): V4 {
        return {
            n: BigInt(prev.n),
        };
    },
};

