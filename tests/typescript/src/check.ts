import {Codec} from "@verilization/runtime";
import {MemoryFormatReader, MemoryFormatWriter} from "./MemoryFormat.js";

export type SimpleValue = number | bigint | string | null | { readonly [name: string]: SimpleValue | undefined };

function equalObj(a: SimpleValue, b: SimpleValue): boolean {
    if(typeof a === "number" && typeof b === "number") {
        return a === b;
    }
    else if(typeof a === "bigint" && typeof b === "bigint") {
        return a === b;
    }
    else if(typeof a === "string" && typeof b === "string") {
        return a === b;
    }
    else if(a === null && b === null) {
        return true;
    }
    else if(typeof a === "object" && typeof b === "object") {
        if(a === null || b === null) {
            return false;
        }

        for(const key in a) {
            if(!Object.prototype.hasOwnProperty.call(a, key)) continue;
            const a1 = a[key];
            const b1 = b[key];

            if(a1 === undefined || b1 === undefined) {
                return false;
            }

            if(!equalObj(a1, b1)) {
                return false;
            }
        }

        for(const key in b) {
            if(!Object.prototype.hasOwnProperty.call(b, key)) continue;
            if(!(key in a)) {
                return false;
            }
        }

        return true;
    }
    else {
        return false;
    }
}

function equalArr(a: Uint8Array, b: Uint8Array): boolean {
    if(a.length !== b.length) {
        return false;
    }

    for(let i = 0; i < a.length; ++i) {
        if(a[i] !== b[i]) {
            return false;
        }
    }

    return true;
}


export async function check<T extends SimpleValue>(codec: Codec<T>, value: T, encoded: Uint8Array): Promise<void> {
    {
        const writer = new MemoryFormatWriter();
        await codec.write(writer, value);
        if(!equalArr(writer.toUint8Array(), encoded)) {
            throw new Error("Encode failed");
        }
    }

    {
        const reader = new MemoryFormatReader(encoded);
        const decoded = await codec.read(reader);
        if(!reader.isEOF()) {
            throw new Error("Decode failed: Did not consume all input");
        }

        if(!equalObj(value, decoded)) {
            throw new Error("Decode failed");
        }
    }
}