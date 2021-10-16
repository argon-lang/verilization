package dev.argon.verilization.runtime;

import java.io.IOException;
import java.util.function.Function;

public interface RemoteConnection {
    <T> T readObject(FormatReader reader, Function<RemoteObjectId, T> createRemoteWrapper) throws IOException;
    <T> void writeObject(FormatWriter writer, T value) throws IOException;

    <T> T invokeMethod(RemoteObjectId id, java.lang.String name, MethodArgument<?>[] arguments, Codec<T> resultCodec);


    public static record MethodArgument<T>(T value, Codec<T> codec) {}
}
