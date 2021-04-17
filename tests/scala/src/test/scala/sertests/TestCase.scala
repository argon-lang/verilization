package sertests

import dev.argon.verilization.scala_runtime.Codec
import zio.Chunk

final case class TestCase[T](codec: Codec[T], value: T, encoded: Chunk[Byte])

