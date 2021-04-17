
name := "verilization-runtime-java"
organization := "dev.argon"
version := "0.1.0-SNAPSHOT"
autoScalaLibrary := false
crossPaths := false

javacOptions ++= Seq(
    "-target", "11",
    "-encoding", "UTF-8",
)
