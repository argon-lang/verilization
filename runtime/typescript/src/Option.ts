import { Converter } from "./Converter.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";
import { Codec } from "./Codec.js";

export type Option<A> = { readonly value: A; } | null;

export function fromCaseSome<A>(value: A): Option<A> {
    return { value };
}

export function fromCaseNone<A>(): Option<A> {
    return null;
}

export function converter<A, B>(elementConverter: Converter<A, B>): Converter<Option<A>, Option<B>> {
    return {
        convert(prev: Option<A>): Option<B> {
            if(prev === null) {
                return null;
            }
            else {
                return { value: elementConverter.convert(prev.value) };
            }
        }
    };
}

export function codec<A>(elementCodec: Codec<A>): Codec<Option<A>> {
    return {
        async read(reader: FormatReader): Promise<Option<A>> {
            const present = await reader.readU8();

            if(present !== 0) {
                const value = await elementCodec.read(reader);
                return { value };
            }
            else {
                return null;
            }
        },

        async write(writer: FormatWriter, value: Option<A>): Promise<void> {
            if(value !== null) {
                await writer.writeU8(1);
                await elementCodec.write(writer, value.value);
            }
            else {
                await writer.writeU8(0);
            }
        },
    };
}
