# Build notes

This environment does not contain an Android SDK/Gradle/Rust toolchain, so an APK cannot be
compiled here. The complete source project is included.

On the development machine:

    cargo install cargo-ndk
    rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
    gradle :app:assembleDebug

Or open the folder in Android Studio and build `app`.
