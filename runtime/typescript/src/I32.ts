import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";

export type I32 = number;

export const codec: Codec<I32> = {
    async read(reader: FormatReader): Promise<I32> {
        return await reader.readU32() | 0;
    },

    write(writer: FormatWriter, value: I32): Promise<void> {
        return writer.writeU32(value >>> 0);
    },
};

export function fromInteger(n: bigint): I32 {
    return Number(n) | 0;
}
