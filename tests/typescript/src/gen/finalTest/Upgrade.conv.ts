import {V3, V4} from "./Upgrade.js";

export function v3_to_v4(prev: V3): V4 {
    return {
        n: BigInt(prev.n),
    };
}

