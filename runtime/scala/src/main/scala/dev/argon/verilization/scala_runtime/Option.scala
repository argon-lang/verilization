package dev.argon.verilization.scala_runtime

import zio.ZIO
import scala.{Option => SOption}

object Option {
    def converter[A, B](elementConverter: Converter[A, B]): Converter[SOption[A], SOption[B]] = elementConverter match {
        case elementConverter: IdentityConverter[A] => new IdentityConverter[SOption[A]]
        case _ => new Converter[SOption[A], SOption[B]] {
            override def convert(prev: SOption[A]): SOption[B] = prev.map(elementConverter.convert)
        }
    }
}
