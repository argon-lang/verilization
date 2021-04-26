import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";

export type U8 = number;

export const codec: Codec<U8> = {
    read(reader: FormatReader): Promise<U8> {
        return reader.readU8();
    },

    write(writer: FormatWriter, value: U8): Promise<void> {
        return writer.writeU8(value);
    },
};
