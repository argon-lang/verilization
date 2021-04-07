import {V1, V2} from "./Referenced.js";


export function v1_to_v2(v1: V1): V2 {
    return {
        x: BigInt(v1.x),
    }
}
