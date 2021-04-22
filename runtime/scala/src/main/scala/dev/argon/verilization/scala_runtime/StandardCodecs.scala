package dev.argon.verilization.scala_runtime

import zio.{IO, ZIO, Chunk}
import java.nio.charset.StandardCharsets

object StandardCodecs {
    
    val natCodec: Codec[BigInt] = new Codec[BigInt] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, BigInt] =
            VLQ.decodeVLQ(reader, false)

        override def write[R, E](writer: FormatWriter[R, E], value: BigInt): ZIO[R, E, Unit] =
            VLQ.encodeVLQ(writer, false, value)
    }
    
    val intCodec: Codec[BigInt] = new Codec[BigInt] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, BigInt] =
            VLQ.decodeVLQ(reader, true)

        override def write[R, E](writer: FormatWriter[R, E], value: BigInt): ZIO[R, E, Unit] =
            VLQ.encodeVLQ(writer, true, value)
    }
    
    val i8Codec: Codec[Byte] = new Codec[Byte] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, Byte] =
            reader.readByte()

        override def write[R, E](writer: FormatWriter[R, E], value: Byte): ZIO[R, E, Unit] =
            writer.writeByte(value)
    }
    
    val i16Codec: Codec[Short] = new Codec[Short] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, Short] =
            reader.readShort()

        override def write[R, E](writer: FormatWriter[R, E], value: Short): ZIO[R, E, Unit] =
            writer.writeShort(value)
    }
    
    val i32Codec: Codec[Int] = new Codec[Int] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, Int] =
            reader.readInt()

        override def write[R, E](writer: FormatWriter[R, E], value: Int): ZIO[R, E, Unit] =
            writer.writeInt(value)
    }
    
    val i64Codec: Codec[Long] = new Codec[Long] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, Long] =
            reader.readLong()

        override def write[R, E](writer: FormatWriter[R, E], value: Long): ZIO[R, E, Unit] =
            writer.writeLong(value)
    }

    val stringCodec: Codec[String] = new Codec[String] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, String] =
            natCodec.read(reader).flatMap { length =>
                if(length > Int.MaxValue)
                    IO.die(new ArithmeticException("Length of string would overflow"))
                else
                    reader.readBytes(length.toInt)
            }.map { data => new String(data.toArray, StandardCharsets.UTF_8) }

        override def write[R, E](writer: FormatWriter[R, E], value: String): ZIO[R, E, Unit] = {
            val data = value.getBytes(StandardCharsets.UTF_8)
            natCodec.write(writer, value.length) *> writer.writeBytes(Chunk.fromArray(data))
        }
            
    }

    def listCodec[A](elementCodec: Codec[A]): Codec[Chunk[A]] = new Codec[Chunk[A]] {

        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, Chunk[A]] =
            natCodec.read(reader).flatMap { length =>
                if(length > Int.MaxValue)
                    IO.succeed(length.toInt)
                else
                    IO.die(new ArithmeticException("Length of chunk would overflow"))
            }.flatMap { length =>
                def readElements(num: Int, data: Chunk[A]): ZIO[R, E, Chunk[A]] =
                    if(num > 0) elementCodec.read(reader).flatMap { elem => readElements(num - 1, data :+ elem) }
                    else IO.succeed(data)

                readElements(length, Chunk.empty)
            }

        override def write[R, E](writer: FormatWriter[R, E], value: Chunk[A]): ZIO[R, E, Unit] =
            natCodec.write(writer, value.size) *> ZIO.foreach_(value) { elem => elementCodec.write(writer, elem) }
            
    }

}
