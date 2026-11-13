"""Package the Temivu Android showcase using the Android SDK and Rust."""
import argparse
import os
from pathlib import Path
import shutil
import tempfile
import zipfile

import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from notices import ROOT, collect_notices, run

TARGETS = {
    "aarch64-linux-android": "arm64-v8a",
    "x86_64-linux-android": "x86_64",
}
FEATURES = ["--locked", "--features", "android-showcase"]



def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--keystore", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    sdk = Path(os.environ["ANDROID_HOME"])
    ndk = Path(os.environ.get("ANDROID_NDK_HOME", sdk / "ndk/27.1.12297006"))
    llvm = ndk / "toolchains/llvm/prebuilt/linux-x86_64/bin"
    tools = sdk / "build-tools/36.0.0"
    target_dir = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")).resolve()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target_dir))
    for target in TARGETS:
        linker = str(llvm / f"{target}26-clang")
        env[f"CARGO_TARGET_{target.replace('-', '_').upper()}_LINKER"] = linker
        env[f"CC_{target.replace('-', '_')}"] = linker
        env[f"AR_{target.replace('-', '_')}"] = str(llvm / "llvm-ar")
    env["RUSTFLAGS"] = (
        env.get("RUSTFLAGS", "") + " -C link-arg=-Wl,-z,max-page-size=16384"
    ).strip()
    run("cargo", "build", *FEATURES, "--example", "showcase_android",
        *(value for target in TARGETS for value in ("--target", target)), env=env)

    with tempfile.TemporaryDirectory(dir=output.parent) as work:
        stage = Path(work)
        resource = stage / "res/drawable-nodpi"
        resource.mkdir(parents=True)
        shutil.copyfile(ROOT / "branding/temivu-mark-512.png", resource / "temivu.png")
        notices = stage / "assets/licenses"
        notices.mkdir(parents=True)
        collect_notices(notices, TARGETS, "android-showcase")
        ndk_notices = notices / "android-ndk"
        ndk_notices.mkdir()
        for name in ("NOTICE", "NOTICE.toolchain"):
            shutil.copyfile(ndk / name, ndk_notices / name)
        compiled = stage / "resources.zip"
        unsigned = stage / "unsigned.apk"
        aligned = stage / "aligned.apk"
        run(tools / "aapt2", "compile", "--dir", stage / "res", "-o", compiled)
        run(tools / "aapt2", "link", "-I", sdk / "platforms/android-34/android.jar",
            "--manifest", Path(__file__).with_name("AndroidManifest.xml"),
            "-A", stage / "assets", "-o", unsigned, compiled)
        with zipfile.ZipFile(unsigned, "a", compression=zipfile.ZIP_STORED) as apk:
            for target, abi in TARGETS.items():
                name = "libshowcase_android.so"
                lib = target_dir / target / "debug/examples" / name
                stripped = stage / name
                run(llvm / "llvm-strip", "--strip-unneeded", "-o", stripped, lib)
                apk.write(stripped, f"lib/{abi}/{name}")
        run(tools / "zipalign", "-P", "16", "-f", "4", unsigned, aligned)
        run(tools / "apksigner", "sign", "--ks", args.keystore.resolve(),
            "--ks-key-alias", "androiddebugkey", "--ks-pass", "pass:android",
            "--key-pass", "pass:android", "--v4-signing-enabled", "false",
            "--out", output, aligned)
    run(tools / "zipalign", "-c", "-P", "16", "4", output)
    run(tools / "apksigner", "verify", "--verbose", output)


if __name__ == "__main__":
    main()
