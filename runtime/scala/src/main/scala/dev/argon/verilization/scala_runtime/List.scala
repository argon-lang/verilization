package dev.argon.verilization.scala_runtime

import zio.{ZIO, IO, Chunk, ChunkBuilder}

object List {
    def fromSequence[A](seq: A*): Chunk[A] = Chunk.fromIterable(seq)

    def converter[A, B](elementConverter: Converter[A, B]): Converter[Chunk[A], Chunk[B]] = elementConverter match {
        case elementConverter: IdentityConverter[A] => new IdentityConverter[Chunk[A]]
        case _ => new Converter[Chunk[A], Chunk[B]] {
            override def convert(prev: Chunk[A]): Chunk[B] = prev.map(elementConverter.convert)
        }
    }

    def codec[A](elementCodec: Codec[A]): Codec[Chunk[A]] = new Codec[Chunk[A]] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, Chunk[A]] =
            Nat.codec.read(reader).flatMap { length =>
                IO.effectTotal { length.bigInteger.intValueExact }
            }.flatMap { length =>
                IO.effectTotal { ChunkBuilder.make[A](length) }.flatMap { chunkBuilder =>
                    elementCodec.read(reader)
                        .flatMap { a => IO.effectTotal { chunkBuilder.addOne(a) } }
                        .repeatN(length)
                        .flatMap { _ =>
                            IO.effectTotal { chunkBuilder.result() }
                        }
                }
            }

        override def write[R, E](writer: FormatWriter[R, E], value: Chunk[A]): ZIO[R, E, Unit] =
            Nat.codec.write(writer, value.size) *> ZIO.foreach_(value) { elem => elementCodec.write(writer, elem) }
    }
}
