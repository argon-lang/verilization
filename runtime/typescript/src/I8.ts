import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";

export type I8 = number;

export const codec: Codec<I8> = {
    async read(reader: FormatReader): Promise<I8> {
        return (await reader.readU8() << 24) >> 24;
    },

    write(writer: FormatWriter, value: I8): Promise<void> {
        return writer.writeU8(value & 0xFF);
    },
};

export function fromInteger(n: bigint): I8 {
    return (Number(n) << 24) >> 24;
}
