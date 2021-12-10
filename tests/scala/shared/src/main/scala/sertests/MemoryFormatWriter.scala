package sertests

import zio.{IO, UIO, Chunk}
import dev.argon.verilization.scala_runtime.FormatWriter
import java.io.ByteArrayOutputStream

final class MemoryFormatWriter private(stream: ByteArrayOutputStream) extends FormatWriter[Any, Nothing] {
    override def writeByte(b: Byte): UIO[Unit] =
        IO.succeed { stream.write(b) }

    override def writeShort(s: Short): UIO[Unit] =
        IO.succeed {
            stream.write(s.toByte)
            stream.write((s >>> 8).toByte)
        }

    override def writeInt(i: Int): UIO[Unit] =
        IO.succeed {
            stream.write(i.toByte)
            stream.write((i >>> 8).toByte)
            stream.write((i >>> 16).toByte)
            stream.write((i >>> 24).toByte)
        }

    override def writeLong(l: Long): UIO[Unit] =
        IO.succeed {
            stream.write(l.toByte)
            stream.write((l >>> 8).toByte)
            stream.write((l >>> 16).toByte)
            stream.write((l >>> 24).toByte)
            stream.write((l >>> 32).toByte)
            stream.write((l >>> 40).toByte)
            stream.write((l >>> 48).toByte)
            stream.write((l >>> 56).toByte)
        }

    override def writeBytes(data: Chunk[Byte]): UIO[Unit] =
        IO.succeed {
            stream.write(data.toArray)
        }

    def toChunk: UIO[Chunk[Byte]] =
        IO.succeed { Chunk.fromArray(stream.toByteArray()) }
}

object MemoryFormatWriter {
    def make: UIO[MemoryFormatWriter] =
        IO.succeed {
            new MemoryFormatWriter(new ByteArrayOutputStream())
        }
}
