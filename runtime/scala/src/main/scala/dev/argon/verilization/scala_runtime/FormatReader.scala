package dev.argon.verilization.scala_runtime

import zio.{ZIO, Chunk}

trait FormatReader[-R, +E] {
    def readByte(): ZIO[R, E, Byte]
    def readShort(): ZIO[R, E, Short]
    def readInt(): ZIO[R, E, Int]
    def readLong(): ZIO[R, E, Long]
    def readBytes(count: Int): ZIO[R, E, Chunk[Byte]]
}
