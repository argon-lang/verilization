package dev.argon.verilization.scala_runtime

class RemoteObject[R, E](
  protected val remote_connection: RemoteConnection[R, E],
  protected val object_id: RemoteObjectId,
)
