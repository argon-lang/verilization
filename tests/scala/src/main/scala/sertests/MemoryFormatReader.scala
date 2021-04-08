package sertests

import zio.{IO, UIO, Chunk}
import dev.argon.verilization.scala_runtime.FormatReader
import java.io.{IOException, EOFException}
import java.nio.{BufferUnderflowException, ByteBuffer, ByteOrder}

final class MemoryFormatReader(data: ByteBuffer) extends FormatReader[Any, EOFException] {

    private def catchErrors(ex: Throwable): IO[EOFException, Nothing] =
        ex match {
            case ex: BufferUnderflowException => IO.fail(new EOFException())
            case _ => IO.die(ex)
        }

    override def readByte(): IO[EOFException, Byte] =
        IO.effect { data.get() }.catchAll(catchErrors)

    override def readShort(): IO[EOFException, Short] =
        IO.effect { data.getShort() }.catchAll(catchErrors)

    override def readInt(): IO[EOFException, Int] =
        IO.effect { data.getInt() }.catchAll(catchErrors)

    override def readLong(): IO[EOFException, Long] =
        IO.effect { data.getLong() }.catchAll(catchErrors)

    override def readBytes(count: Int): IO[EOFException, Chunk[Byte]] =
        IO.effect {
            val buffer = new Array[Byte](count)
            data.get(buffer)
            Chunk.fromArray(buffer)
        }.catchAll(catchErrors)

    def isEOF: UIO[Boolean] =
        IO.effectTotal { !data.hasRemaining() }
}
