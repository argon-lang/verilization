package dev.argon.verilization.scala_runtime

import zio.{ZIO, IO}
import scala.{Option => SOption}

object Option {
    def fromCaseSome[A](value: A): Option[A] = Some(value)
    def fromCaseNone[A](): Option[A] = None

    def converter[A, B](elementConverter: Converter[A, B]): Converter[SOption[A], SOption[B]] = elementConverter match {
        case _: IdentityConverter[_] => new IdentityConverter[SOption[A]]
        case _ => new Converter[SOption[A], SOption[B]] {
            override def convert(prev: SOption[A]): SOption[B] = prev.map(elementConverter.convert)
        }
    }

    def codec[A](elementCodec: Codec[A]): Codec[SOption[A]] = new Codec[SOption[A]] {
        override def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, SOption[A]] =
            U8.codec.read(reader).flatMap { present =>
                if(present != 0) {
                    elementCodec.read(reader).map(Some.apply)
                }
                else {
                    IO.none
                }
            }

        override def write[R, E](writer: FormatWriter[R, E], value: SOption[A]): ZIO[R, E, Unit] =
            U8.codec.write(writer, if(value.isDefined) 1 else 0) *> ZIO.foreachDiscard(value) { elem => elementCodec.write(writer, elem) }
    }
}
