# iOS native development package

The [builder](../../scripts/build_ios_native.py) packages the existing Rust C ABI
as a dynamic `HomeVerb.framework` for arm64 iOS devices and a separate universal
arm64/x86_64 iOS Simulator framework. Xcode combines them into an XCFramework.
The canonical C header is copied from `crates/ffi/include/homeverb.h`; this package
owns only its module map and the development packaging process, not Flutter views.

Use the [progressive iOS guide](../../docs/development/09-ios-native.md) for the
private Keychain build group, build command, embedding and acceptance gates.
The builder requires macOS, Xcode, Python 3.11+, the three installed Rust targets
and cached Cargo dependencies. It builds release libraries offline and never
downloads a dependency, installs an app or publishes a package.

The output is `target/ios/homeverb-core.xcframework.zip`. Only a package whose
extracted binaries pass platform/architecture, deployment, dependency, install-name,
export, metadata, canonical-header and local-signature checks replaces the prior
archive. A C module consumer must link every public function for all three target
architectures. Device and Simulator binaries are never combined by `lipo`.

Frameworks receive local ad-hoc seals using the pseudo-identity `-` and no timestamp
server. These prove local integrity, not author identity, device provisioning or
distribution signing. The app must embed and sign the selected framework using its
own reviewed signing configuration. The Keychain group is compiled into the Rust
library and must match that app's private application identifier; changing it
requires rebuilding, not editing a runtime preference.

The device deployment target is iOS 13.0. Simulator builds require iOS 14.0,
consistent with the Rust arm64 Simulator target's minimum. These are compilation
targets, not physical-device support claims. App/Flutter loading, actual Keychain
behavior, lifecycle and appliance operations need their own acceptance evidence.
The project license and release decision remain pending.

`acceptance/main.m` is a separate test-only Simulator app, built and run by
`scripts/verify_ios_acceptance.py`. It embeds this production framework and links
the Simulator-only `ios_store_probe` Rust example for fictional Keychain checks.
Neither test source nor test export is packaged in the XCFramework. The runner
creates and deletes its own Simulator and requires fresh reports from two distinct
app processes. See the guide for the exact command and physical-device limitations.

The package format follows Apple's [XCFramework guidance](https://developer.apple.com/documentation/xcode/creating-a-multi-platform-binary-framework-bundle).
Apple distinguishes [ad-hoc seals](https://developer.apple.com/documentation/security/seccodesignatureflags/adhoc)
from signatures carrying an identity. Both references were inspected October 16, 2026.
