


lazy val root = project.in(file("."))
    .aggregate(scalaRuntime.js, scalaRuntime.jvm)
    .settings(
        publish := {},
        publishLocal := {},
        crossScalaVersions := Nil,
    )


lazy val scalaRuntime = crossProject(JSPlatform, JVMPlatform).in(file("."))
    .settings(
        name := "verilization-runtime-scala",
        organization := "dev.argon",
        organizationName := "Argon",
        organizationHomepage := Some(url("https://github.com/argon-lang")),
        version := "0.1.0",
        description := "Runtime library for verilization serializers.",
        homepage := Some(url("https://github.com/argon-lang/verilization")),
        licenses := Seq("Apache 2.0" -> url("http://www.apache.org/licenses/LICENSE-2.0.txt")),

        scmInfo := Some(
            ScmInfo(
                url("https://github.com/argon-lang/verilization"),
                "scm:git@github.com:argon-lang/verilization"
            )
        ),

        developers := List(
            Developer(
                id = "argon-dev",
                name = "argon-dev",
                email = "argon@argon.dev",
                url = url("https://github.com/argon-dev"),
            )
        ),


        libraryDependencies += "dev.zio" %%% "zio" % "1.0.5",
    )
    .jvmSettings(
        crossScalaVersions := List("2.13.5", "2.12.13"),
    )
    .jsSettings(
        crossScalaVersions := List("2.13.5", "2.12.13"),
    )

