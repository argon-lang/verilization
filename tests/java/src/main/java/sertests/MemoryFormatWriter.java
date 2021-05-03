package sertests;

import java.io.ByteArrayOutputStream;
import java.io.IOException;

import dev.argon.verilization.runtime.FormatWriter;

public final class MemoryFormatWriter implements FormatWriter {

    private final ByteArrayOutputStream stream = new ByteArrayOutputStream();

    @Override
	public void writeByte(byte b) throws IOException {
        stream.write(b);
    }

    @Override
	public void writeShort(short s) throws IOException {
        writeByte((byte)s);
        writeByte((byte)(s >>> 8));
    }

    @Override
	public void writeInt(int i) throws IOException {
        writeShort((short)i);
        writeShort((short)(i >>> 16));
    }

    @Override
	public void writeLong(long l) throws IOException {
        writeInt((int)l);
        writeInt((int)(l >>> 32));
    }

    @Override
	public void writeBytes(byte[] data) throws IOException {
        stream.write(data);
    }

    public byte[] toByteArray() {
        return stream.toByteArray();
    }

}
