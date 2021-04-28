import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";
import { encodeVLQ, decodeVLQ } from "./VLQ.js";

export type Nat = bigint;

export const codec: Codec<Nat> = {
    read(reader: FormatReader): Promise<Nat> {
        return decodeVLQ(reader, false);
    },

    write(writer: FormatWriter, value: Nat): Promise<void> {
        return encodeVLQ(writer, false, value);
    },
};

export function fromInteger(n: bigint): Nat {
    return n;
}
