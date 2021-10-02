
name := "verilization-runtime"
organization := "dev.argon"
organizationName := "Argon"
organizationHomepage := Some(url("https://github.com/argon-lang"))
version := "0.1.0"
description := "Runtime library for verilization serializers."
homepage := Some(url("https://github.com/argon-lang/verilization"))
licenses := Seq("Apache 2.0" -> url("http://www.apache.org/licenses/LICENSE-2.0.txt"))

scmInfo := Some(
    ScmInfo(
        url("https://github.com/argon-lang/verilization"),
        "scm:git@github.com:argon-lang/verilization"
    )
)

developers := List(
    Developer(
        id = "argon-dev",
        name = "argon-dev",
        email = "argon@argon.dev",
        url = url("https://github.com/argon-dev"),
    )
)

autoScalaLibrary := false
crossPaths := false

javacOptions ++= Seq(
    "--release", "11",
    "-encoding", "UTF-8",
    "-Xlint:unchecked"
)
