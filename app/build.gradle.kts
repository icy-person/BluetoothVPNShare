plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.example.bluetoothvpnshare"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.example.bluetoothvpnshare"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "1.0.0"
    }

    buildTypes { release { isMinifyEnabled = false } }
    sourceSets["main"].jniLibs.srcDirs("src/main/jniLibs")
    packaging.jniLibs.useLegacyPackaging = true
}

tasks.register<Exec>("buildRust") {
    workingDir(project.projectDir.resolve("../rust"))
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "x86_64",
        "-o", project.projectDir.resolve("src/main/jniLibs").absolutePath,
        "build", "--release"
    )
}

tasks.named("preBuild").configure { dependsOn("buildRust") }
