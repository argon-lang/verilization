package dev.argon.verilization.scala_runtime

import zio.{ZIO, Chunk}
import scala.Predef.{String => SString}
import java.nio.charset.StandardCharsets

object String {
    def fromString(s: SString): String = s

    val codec: Codec[SString] = new Codec[SString] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, SString] =
            Nat.codec.read(reader).flatMap { length =>
                reader.readBytes(length.bigInteger.intValueExact)
            }.map { data => new SString(data.toArray, StandardCharsets.UTF_8) }

        override def write[R, E](writer: FormatWriter[R, E], value: SString): ZIO[R, E, Unit] = {
            val data = value.getBytes(StandardCharsets.UTF_8)
            Nat.codec.write(writer, value.length) *> writer.writeBytes(Chunk.fromArray(data))
        }
            
    }
}
