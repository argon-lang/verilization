import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";

export type U16 = number;

export const codec: Codec<U16> = {
    read(reader: FormatReader): Promise<U16> {
        return reader.readU16();
    },

    write(writer: FormatWriter, value: U16): Promise<void> {
        return writer.writeU16(value);
    },
};
