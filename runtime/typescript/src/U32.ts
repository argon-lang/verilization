import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";

export type U32 = number;

export const codec: Codec<U32> = {
    read(reader: FormatReader): Promise<U32> {
        return reader.readU32();
    },

    write(writer: FormatWriter, value: U32): Promise<void> {
        return writer.writeU32(value);
    },
};
