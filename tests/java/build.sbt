
lazy val runtimeLib = RootProject(file("../../runtime/java"))

lazy val proj = project.in(file("."))
    .dependsOn(runtimeLib)
    .settings(
        name := "verilization-tests",
        organization := "dev.argon",
        crossPaths := false,
        unmanagedSourceDirectories in Compile += baseDirectory.value / "gen",

        javacOptions ++= Seq(
            "-target", "11",
            "-encoding", "UTF-8",
        ),
    )