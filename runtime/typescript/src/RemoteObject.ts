import { RemoteConnection } from "./RemoteConnection.js";
import { RemoteObjectId } from "./RemoteObjectId.js";

export interface RemoteObject {
    readonly [RemoteObject.connectionSymbol]: RemoteConnection;
    readonly [RemoteObject.objectIdSymbol]: RemoteObjectId;
}

export namespace RemoteObject {
    export const connectionSymbol: unique symbol = Symbol();
    export const objectIdSymbol: unique symbol = Symbol();
}
