package dev.argon.verilization.scala_runtime

import zio.{IO, ZIO, Chunk, ChunkBuilder}

import scala.{Int => SInt}

object Nat {
    def fromInteger(i: SInt): Nat = i.abs
    def fromInteger(l: Long): Nat = l.abs
    def fromInteger(i: BigInt): Nat = i.abs

    val codec: Codec[BigInt] = new Codec[BigInt] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, BigInt] =
            VLQ.decodeVLQ(reader, false)

        override def write[R, E](writer: FormatWriter[R, E], value: BigInt): ZIO[R, E, Unit] =
            VLQ.encodeVLQ(writer, false, value)
    }
}

object Int {
    def fromInteger(i: SInt): Nat = i
    def fromInteger(l: Long): Nat = l
    def fromInteger(i: BigInt): Nat = i

    val codec: Codec[BigInt] = new Codec[BigInt] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, BigInt] =
            VLQ.decodeVLQ(reader, true)

        override def write[R, E](writer: FormatWriter[R, E], value: BigInt): ZIO[R, E, Unit] =
            VLQ.encodeVLQ(writer, true, value)
    }
}

object I8 {
    def fromInteger(i: SInt): I8 = i.toByte

    val codec: Codec[Byte] = new Codec[Byte] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, Byte] =
            reader.readByte()

        override def write[R, E](writer: FormatWriter[R, E], value: Byte): ZIO[R, E, Unit] =
            writer.writeByte(value)
    }
}

object U8 {
    def fromInteger(i: SInt): U8 = i.toByte

    val codec: Codec[Byte] = I8.codec
}

object I16 {
    def fromInteger(i: SInt): I32 = i

    val codec: Codec[Short] = new Codec[Short] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, Short] =
            reader.readShort()

        override def write[R, E](writer: FormatWriter[R, E], value: Short): ZIO[R, E, Unit] =
            writer.writeShort(value)
    }
}

object U16 {
    def fromInteger(i: SInt): I16 = i.toShort

    val codec: Codec[Short] = I16.codec
}

object I32 {
    def fromInteger(i: SInt): I32 = i

    val codec: Codec[SInt] = new Codec[SInt] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, SInt] =
            reader.readInt()

        override def write[R, E](writer: FormatWriter[R, E], value: SInt): ZIO[R, E, Unit] =
            writer.writeInt(value)
    }
}

object U32 {
    def fromInteger(i: SInt): U32 = i
    def fromInteger(l: Long): U32 = l.toInt

    val codec: Codec[SInt] = I32.codec
}

object I64 {
    def fromInteger(i: SInt): I64 = i
    def fromInteger(l: Long): I64 = l

    val codec: Codec[Long] = new Codec[Long] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, Long] =
            reader.readLong()

        override def write[R, E](writer: FormatWriter[R, E], value: Long): ZIO[R, E, Unit] =
            writer.writeLong(value)
    }
}

object U64 {
    def fromInteger(i: SInt): U64 = i
    def fromInteger(l: Long): U64 = l
    def fromInteger(i: BigInt): U64 = i.toLong

    val codec: Codec[Long] = I64.codec
}

