package sertests

import zio.{IO, UIO, Chunk}
import dev.argon.verilization.scala_runtime.FormatReader
import java.io.{IOException, EOFException}
import java.nio.{BufferUnderflowException, ByteBuffer, ByteOrder}

final class MemoryFormatReader private(data: ByteBuffer) extends FormatReader[Any, EOFException] {

    private def catchErrors(ex: Throwable): IO[EOFException, Nothing] =
        ex match {
            case ex: BufferUnderflowException => IO.fail(new EOFException())
            case _ => IO.die(ex)
        }

    override def readByte(): IO[EOFException, Byte] =
        IO.attempt { data.get() }.catchAll(catchErrors)

    override def readShort(): IO[EOFException, Short] =
        IO.attempt { data.getShort() }.catchAll(catchErrors)

    override def readInt(): IO[EOFException, Int] =
        IO.attempt { data.getInt() }.catchAll(catchErrors)

    override def readLong(): IO[EOFException, Long] =
        IO.attempt { data.getLong() }.catchAll(catchErrors)

    override def readBytes(count: Int): IO[EOFException, Chunk[Byte]] =
        IO.attempt {
            val buffer = new Array[Byte](count)
            data.get(buffer)
            Chunk.fromArray(buffer)
        }.catchAll(catchErrors)

    def isEOF: UIO[Boolean] =
        IO.succeed { !data.hasRemaining() }
}

object MemoryFormatReader {
    def fromChunk(chunk: Chunk[Byte]): UIO[MemoryFormatReader] =
        IO.succeed {
            new MemoryFormatReader(ByteBuffer.wrap(chunk.toArray).order(ByteOrder.LITTLE_ENDIAN))
        }
}
