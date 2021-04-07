import {FormatWriter, FormatReader} from "./FormatIO.js";

export interface Codec<T> {
    read(reader: FormatReader): Promise<T>;
    write(writer: FormatWriter, value: T): Promise<void>;
}
