
import {V3, V4} from "./Pair.js";
import {Converter} from "@verilization/runtime";

export function v3_to_v4<A_1, A_2, B_1, B_2>(a_conv: Converter<A_1, A_2>, b_conv: Converter<B_1, B_2>): Converter<V3<A_1, B_1>, V4<A_2, B_2>> {
    return {
        convert(prev: V3<A_1, B_1>): V4<A_2, B_2> {
            return {
                left: a_conv.convert(prev.left),
                right: b_conv.convert(prev.right),
                other: "dummy",
            };
        },
    };
};

