package dev.argon.verilization.scala_runtime

import zio.ZIO

trait Codec[A] {
    def read[R, E](reader: FormatReader[R, E]): ZIO[R, E, A]
    def write[R, E](writer: FormatWriter[R, E], value: A): ZIO[R, E, Unit]
}
