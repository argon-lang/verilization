import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";

export type I16 = number;

export const codec: Codec<I16> = {
    async read(reader: FormatReader): Promise<I16> {
        return (await reader.readU16() << 16) >> 16;
    },

    write(writer: FormatWriter, value: I16): Promise<void> {
        return writer.writeU8(value & 0xFFFF);
    },
};
