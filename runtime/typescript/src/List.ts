import { Converter, IdentityConverter } from "./Converter.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";
import { Codec } from "./Codec.js";
import * as StandardCodecs from "./StandardCodecs.js";

type ValueTypeList<A> =
    A extends number ? (Uint8Array | Int8Array | Uint16Array | Int16Array | Uint32Array | Int32Array) :
    A extends bigint ? (BigUint64Array | BigInt64Array) :
    never;

export type List<A> = readonly A[] | ValueTypeList<A>;

export namespace List {
    export function converter<A, B>(elemConv: Converter<A, B>): Converter<List<A>, List<B>> {
        if(elemConv instanceof IdentityConverter) {
            return Converter.identity<List<A>>() as unknown as Converter<List<A>, List<B>>;
        }

        return {
            convert(prev: List<A>): List<B> {
                const result: B[] = [];
                for(const a of prev) {
                    result.push(elemConv.convert(a as A));
                }
                return result;
            },
        };
    }

    export function codec<A>(elemCodec: Codec<A>): Codec<List<A>> {
        if(elemCodec as unknown === StandardCodecs.u8) {
            return u8list as Codec<List<A>>;
        }
        else if(elemCodec as unknown === StandardCodecs.i8) {
            return i8list as Codec<List<A>>;
        }
        else if(elemCodec as unknown === StandardCodecs.u16) {
            return u16list as Codec<List<A>>;
        }
        else if(elemCodec as unknown === StandardCodecs.i16) {
            return i16list as Codec<List<A>>;
        }
        else if(elemCodec as unknown === StandardCodecs.u32) {
            return u32list as Codec<List<A>>;
        }
        else if(elemCodec as unknown === StandardCodecs.i32) {
            return i32list as Codec<List<A>>;
        }
        else if(elemCodec as unknown === StandardCodecs.u64) {
            return u64list as Codec<List<A>>;
        }
        else if(elemCodec as unknown === StandardCodecs.i64) {
            return i64list as Codec<List<A>>;
        }
        else {
            return otherList(elemCodec);
        }
    }

    const u8list: Codec<Uint8Array> = {
        async read(reader: FormatReader): Promise<Uint8Array> {
            const length = await StandardCodecs.nat.read(reader);
            if(length > BigInt(Number.MAX_SAFE_INTEGER)) {
                throw new Error("Length of array too large");
            }

            const data = await reader.readBytes(Number(length));
            return data;
        },

        async write(writer: FormatWriter, value: Uint8Array): Promise<void> {
            await StandardCodecs.nat.write(writer, BigInt(value.length));
            await writer.writeBytes(value);
        },
    };

    const i8list: Codec<Int8Array> = {
        async read(reader: FormatReader): Promise<Int8Array> {
            return new Int8Array((await u8list.read(reader)).buffer);
        },

        async write(writer: FormatWriter, value: Int8Array): Promise<void> {
            return u8list.write(writer, new Uint8Array(value.buffer));
        },
    };

    const u16list: Codec<Uint16Array> = {
        async read(reader: FormatReader): Promise<Uint16Array> {
            const length = await StandardCodecs.nat.read(reader) * 2n;
            if(length > BigInt(Number.MAX_SAFE_INTEGER)) {
                throw new Error("Length of array too large");
            }

            const data = new Uint16Array(Number(length));
            for(let i = 0; i < data.length; ++i) {
                data[i] = await reader.readU16();
            }
            return data;
        },

        async write(writer: FormatWriter, value: Uint16Array): Promise<void> {
            await StandardCodecs.nat.write(writer, BigInt(value.length));
            for(let i = 0; i < value.length; ++i) {
                await writer.writeU16(value[i]);
            }
        },
    };

    const i16list: Codec<Int16Array> = {
        async read(reader: FormatReader): Promise<Int16Array> {
            return new Int16Array((await u16list.read(reader)).buffer);
        },

        async write(writer: FormatWriter, value: Int16Array): Promise<void> {
            return u16list.write(writer, new Uint16Array(value.buffer));
        },
    };

    const u32list: Codec<Uint32Array> = {
        async read(reader: FormatReader): Promise<Uint32Array> {
            const length = await StandardCodecs.nat.read(reader) * 4n;
            if(length > BigInt(Number.MAX_SAFE_INTEGER)) {
                throw new Error("Length of array too large");
            }

            const data = new Uint32Array(Number(length));
            for(let i = 0; i < data.length; ++i) {
                data[i] = await reader.readU32();
            }
            return data;
        },

        async write(writer: FormatWriter, value: Uint32Array): Promise<void> {
            await StandardCodecs.nat.write(writer, BigInt(value.length));
            for(let i = 0; i < value.length; ++i) {
                await writer.writeU32(value[i]);
            }
        },
    };

    const i32list: Codec<Int32Array> = {
        async read(reader: FormatReader): Promise<Int32Array> {
            return new Int32Array((await u32list.read(reader)).buffer);
        },

        async write(writer: FormatWriter, value: Int32Array): Promise<void> {
            return u32list.write(writer, new Uint32Array(value.buffer));
        },
    };

    const u64list: Codec<BigUint64Array> = {
        async read(reader: FormatReader): Promise<BigUint64Array> {
            const length = await StandardCodecs.nat.read(reader) * 8n;
            if(length > BigInt(Number.MAX_SAFE_INTEGER)) {
                throw new Error("Length of array too large");
            }

            const data = new BigUint64Array(Number(length));
            for(let i = 0; i < data.length; ++i) {
                data[i] = await reader.readU64();
            }
            return data;
        },

        async write(writer: FormatWriter, value: BigUint64Array): Promise<void> {
            await StandardCodecs.nat.write(writer, BigInt(value.length));
            for(let i = 0; i < value.length; ++i) {
                await writer.writeU64(value[i]);
            }
        },
    };

    const i64list: Codec<BigInt64Array> = {
        async read(reader: FormatReader): Promise<BigInt64Array> {
            return new BigInt64Array((await u64list.read(reader)).buffer);
        },

        async write(writer: FormatWriter, value: BigInt64Array): Promise<void> {
            return u64list.write(writer, new BigUint64Array(value.buffer));
        },
    };

    function otherList<T>(elementCodec: Codec<T>): Codec<ReadonlyArray<T>> {
        return {
            async read(reader: FormatReader): Promise<ReadonlyArray<T>> {
                const length = await StandardCodecs.nat.read(reader);
                if(length > BigInt(Number.MAX_SAFE_INTEGER)) {
                    throw new Error("Length of array too large");
                }

                const data = new Array<T>(Number(length));
                for(let i = 0; i < data.length; ++i) {
                    data[i] = await elementCodec.read(reader);
                }
        
                return data;
            },

            async write(writer: FormatWriter, value: ReadonlyArray<T>): Promise<void> {
                await StandardCodecs.nat.write(writer, BigInt(value.length));
                for(let i = 0; i < value.length; ++i) {
                    await elementCodec.write(writer, value[i]);
                }
            },
        };
    }
}
