import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";
import { encodeVLQ, decodeVLQ } from "./VLQ.js";

export type Int = bigint;

export const codec: Codec<Int> = {
    read(reader: FormatReader): Promise<Int> {
        return decodeVLQ(reader, true);
    },

    write(writer: FormatWriter, value: Int): Promise<void> {
        return encodeVLQ(writer, true, value);
    },
};

export function fromInteger(n: bigint): Int {
    return n;
}
