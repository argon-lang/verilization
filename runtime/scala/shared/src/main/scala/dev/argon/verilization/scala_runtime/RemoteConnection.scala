package dev.argon.verilization.scala_runtime

import zio.ZIO
import RemoteConnection.MethodArgument

trait RemoteConnection[R, E] {
  def readObject[T <: AnyRef](createRemoteWrapper: RemoteObjectId => T): ZIO[R, E, T]
  def writeObject[T <: AnyRef](value: T): ZIO[R, E, Unit]

  def invokeMethod[T](objectId: RemoteObjectId, name: java.lang.String, arguments: Seq[MethodArgument[_]], resultCodec: Codec[T]): ZIO[R, E, T]
}

object RemoteConnection {
  final case class MethodArgument[T](value: T, codec: Codec[T])
}
