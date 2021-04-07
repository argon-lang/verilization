
lazy val runtimeLib = RootProject(file("../../runtime/scala"))

lazy val proj = project.in(file("."))
    .dependsOn(runtimeLib)
    .settings(
        name := "verilization-tests",
        organization := "dev.argon",
        crossPaths := false,
        Compile / unmanagedSourceDirectories += baseDirectory.value / "gen",


        scalaVersion := "2.13.5",
        libraryDependencies += "dev.zio" %% "zio" % "1.0.5",

    )
