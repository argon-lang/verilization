package sertests;

import dev.argon.verilization.runtime.FormatReader;
import java.io.IOException;
import java.io.EOFException;
import java.nio.BufferUnderflowException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;

public final class MemoryFormatReader implements FormatReader {

    public MemoryFormatReader(ByteBuffer data) {
        data.order(ByteOrder.LITTLE_ENDIAN);
        this.data = data;
    }

    private final ByteBuffer data;

    @Override
    public byte readByte() throws IOException {
        try {
            return data.get();
        }
        catch(BufferUnderflowException ex) {
            throw new EOFException();
        }
    }

	@Override
    public short readShort() throws IOException {
        try {
            return data.getShort();
        }
        catch(BufferUnderflowException ex) {
            throw new EOFException();
        }
    }

    @Override
	public int readInt() throws IOException {
        try {
            return data.getInt();
        }
        catch(BufferUnderflowException ex) {
            throw new EOFException();
        }
    }

    @Override
	public long readLong() throws IOException {
        try {
            return data.getLong();
        }
        catch(BufferUnderflowException ex) {
            throw new EOFException();
        }
    }

    @Override
	public byte[] readBytes(int count) throws IOException {
        byte[] buffer = new byte[count];
        data.get(buffer);
        return buffer;
    }

    public boolean isEOF() {
        return !data.hasRemaining();
    }
}
