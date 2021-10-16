package dev.argon.verilization.runtime;

public class RemoteObject {

    public RemoteObject(RemoteConnection connection, RemoteObjectId id) {
        this.connection = connection;
        this.id = id;
    }

    protected final RemoteConnection connection;
    protected final RemoteObjectId id;

}
