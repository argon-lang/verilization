package dev.argon.verilization.scala_runtime

import zio.{ZIO, Chunk}
import scala.{Int => SInt}

trait FormatReader[-R, +E] {
    def readByte(): ZIO[R, E, Byte]
    def readShort(): ZIO[R, E, Short]
    def readInt(): ZIO[R, E, SInt]
    def readLong(): ZIO[R, E, Long]
    def readBytes(count: SInt): ZIO[R, E, Chunk[Byte]]
}
