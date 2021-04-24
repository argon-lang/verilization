import {V1, V2} from "./Referenced.js";
import {Converter} from "@verilization/runtime";

export const v1_to_v2: Converter<V1, V2> = {
    convert(v1: V1): V2 {
        return {
            x: BigInt(v1.x),
        }
    },
}; 
