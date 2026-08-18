# Bluetooth VPN Share

Rootless Android proxy for sharing the **currently active Android VPN over Bluetooth PAN**.

This is intentionally modeled after the local-proxy approach documented by VPN Share: the
Android app accepts connections from the tethered client and opens the outbound sockets from
the Android process. Android's existing VPN can therefore route those sockets. VPN Share's
Google Play listing explicitly describes this local-proxy architecture and says it does not
require root. citeturn0search0

## Architecture

    Linux laptop
       │
       │ Bluetooth PAN
       ▼
    Android PAN address:1080
       │
       ▼
    Rust proxy (SOCKS5 + HTTP CONNECT)
       │
       ▼
    Android socket API
       │
       ▼
    existing VPN app
       │
       ▼
    Internet

## Important limitation

A rootless Android app cannot transparently intercept arbitrary IP packets arriving from a
Bluetooth-tethered client. This project therefore exposes a proxy. For Linux you can either
configure individual applications to use SOCKS5/HTTP, or run a local transparent adapter such
as tun2socks on the laptop and point it at the Android SOCKS5 endpoint.

The Android app does **not** call `VpnService.prepare()` and does not create a second VPN. It
also does not call `VpnService.protect()` for outbound sockets: the purpose is for the currently
active third-party VPN to remain eligible to route them. A VPN using per-app rules can still
exclude this app.

## Features

- No root.
- Bluetooth PAN friendly: bind to `0.0.0.0` so the proxy is reachable through the PAN address.
- Rust native core.
- SOCKS5 TCP CONNECT.
- SOCKS5 UDP ASSOCIATE.
- HTTP CONNECT.
- HTTP absolute-form proxy requests for ordinary HTTP.
- Optional username/password authentication for SOCKS5 and HTTP.
- Configurable port.
- Foreground service so Android is less likely to kill the proxy.
- Connection/data counters.
- No packet logging and no DNS logging.

## Build

Install:

    cargo install cargo-ndk
    rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

Open the project in Android Studio, or use a local Gradle installation:

    gradle :app:assembleDebug

The resulting APK is:

    app/build/outputs/apk/debug/app-debug.apk

## Use

1. Pair the laptop and phone.
2. Enable Android Bluetooth tethering/PAN.
3. Start the VPN on the phone and verify the phone itself has the VPN IP.
4. Start this app and choose port `1080`.
5. Find the phone's PAN address on Linux:

       ip addr

6. Test SOCKS5:

       curl --proxy socks5h://PHONE_PAN_IP:1080 https://ifconfig.me

7. Test HTTP CONNECT:

       curl -x http://PHONE_PAN_IP:1080 https://ifconfig.me

For system-wide Linux traffic, use tun2socks or another transparent proxy adapter on the
laptop. The Android side remains rootless.

## Why this can use the existing VPN

The outbound connection is created by the Android application process. Android's VPN routing
can route ordinary application sockets through the active VPN. We deliberately do not protect
those sockets from the VPN. This is the same basic principle as the local-proxy approach
advertised by VPN Share. citeturn0search0

## Compatibility

Some Android vendors/VPNs can impose restrictions on tethering, LocalProxy-like behavior, or
per-app VPN routing. If the proxy works but the public IP is the normal mobile/Wi-Fi IP, inspect
the VPN app's per-app/exclusion settings first.
