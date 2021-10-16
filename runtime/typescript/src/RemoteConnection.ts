import { Codec } from "./Codec.js";
import {FormatReader, FormatWriter} from "./FormatIO.js";
import { RemoteObjectId } from "./RemoteObjectId.js";

export interface RemoteConnection {
    readObject<T>(reader: FormatReader, createRemoteWrapper: (id: RemoteObjectId) => T): Promise<T>;
    writeObject<T>(writer: FormatWriter, value: T): Promise<void>;

    invokeMethod<T>(id: RemoteObjectId, name: string, args: RemoteConnection.MethodArgumentAny[], resultCodec: Codec<T>): Promise<T>;
}

export namespace RemoteConnection {
    export interface MethodArgumentAny {
        withType<A>(f: <T>(arg: MethodArgument<T>) => A): A;
    }

    export interface MethodArgument<T> {
        value: T,
        codec: Codec<T>,
    }

    export function wrapArgument<T>(arg: MethodArgument<T>): MethodArgumentAny {
        return {
            withType<A>(f: <U>(arg: MethodArgument<U>) => A): A {
                return f(arg);
            },
        };
    }
}
