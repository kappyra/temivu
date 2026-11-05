# Android Showcase

The Android build uses the same Temivu showcase as the desktop app. It supports
64-bit ARM and x86 Android 8 or later with OpenGL ES 3.0. The video player
controls can be previewed on Android. Local video decoding in the showcase
uses the Linux backend. Android system screen-reader integration is pending.

On Linux, install Android SDK platform 34, Build Tools 36.0.0 and NDK
27.1.12297006. Set `ANDROID_HOME`, make Java available, and add both Rust targets:

```sh
rustup target add aarch64-linux-android x86_64-linux-android
python3 examples/showcase/android/package.py target/temivu-showcase.apk \
    --keystore "$HOME/.android/debug.keystore"
```

The packaging script expects the standard Android debug keystore and includes
the project, Forui, font and Android-target dependency notices under
`assets/licenses/` in the APK.
