package dev.argon.verilization.scala_runtime

import zio.{ZIO, Chunk}
import scala.{Int => SInt}

trait FormatWriter[-R, +E] {
    def writeByte(b: Byte): ZIO[R, E, Unit]
    def writeShort(s: Short): ZIO[R, E, Unit]
    def writeInt(i: SInt): ZIO[R, E, Unit]
    def writeLong(l: Long): ZIO[R, E, Unit]
    def writeBytes(data: Chunk[Byte]): ZIO[R, E, Unit]
}
