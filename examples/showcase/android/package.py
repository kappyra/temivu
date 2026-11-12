"""Package the Temivu Android showcase using the Android SDK and Rust."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[3]
TARGETS = {
    "aarch64-linux-android": "arm64-v8a",
    "x86_64-linux-android": "x86_64",
}
FEATURES = ["--locked", "--features", "android-showcase"]


def run(*args, capture=False, env=None):
    return subprocess.run(
        args, cwd=ROOT, check=True, text=True, capture_output=capture, env=env
    ).stdout


def collect_notices(destination):
    for name in ("LICENSE", "THIRD_PARTY_NOTICES.md"):
        shutil.copyfile(ROOT / name, destination / name)
    third_party = ROOT / "third_party"
    for directory, pattern in (
        ("forui", "LICENSE*"), ("inter", "OFL.txt"), ("egui-fonts", "*.txt"),
        ("showcase-fonts", "*.txt"),
    ):
        out = destination / "third_party" / directory
        out.mkdir(parents=True)
        matches = sorted((third_party / directory).glob(pattern))
        if not matches:
            raise ValueError(f"Missing notice: {directory}/{pattern}")
        for path in matches:
            shutil.copyfile(path, out / path.name)

    upstream = third_party / "upstream-licenses"
    provenance = json.loads((upstream / "provenance.json").read_text())
    recorded = {(p["name"], p["version"]): p["files"] for p in provenance["packages"]}
    packages = {}
    linked = set()
    for target in TARGETS:
        flags = [*FEATURES, "--target", target]
        tree = run("cargo", "tree", *flags, "--edges", "normal,build", "--prefix",
                   "none", "--format", "{p}", capture=True)
        for line in tree.splitlines():
            match = re.match(r"^(\S+) v(\S+)", line)
            if match:
                linked.add(match.groups())
        metadata = json.loads(run("cargo", "metadata", *FEATURES, "--format-version",
                                  "1", "--filter-platform", target, capture=True))
        for package in metadata["packages"]:
            packages[(package["name"], package["version"])] = package

    index = []
    for key in sorted(linked):
        package = packages[key]
        if package["source"] is None:
            continue
        folder = Path(package["manifest_path"]).parent
        notices = {
            f for f in folder.iterdir()
            if f.is_file() and f.name.lower().startswith(
                ("license", "licence", "copying", "copyright", "notice", "unlicense")
            )
        }
        if package["license_file"]:
            notices.add(folder / package["license_file"])
        if not notices:
            for record in recorded.get(key, []):
                file = upstream / record["path"]
                if hashlib.sha256(file.read_bytes()).hexdigest() != record["sha256"]:
                    raise ValueError(f"Changed upstream notice: {file}")
                notices.add(file)
        if not notices:
            raise ValueError(f"Missing license text for {key[0]} {key[1]}")
        notices.update(f for f in folder.iterdir() if f.is_file() and f.name in
                       {"AUTHORS", "AUTHORS.txt", "CONTRIBUTORS", "CONTRIBUTORS.txt"})
        label = "-".join(key)
        index.append(f"{label}: {package['license']} ({package['repository'] or package['source']})")
        directory = destination / "crates" / label
        directory.mkdir(parents=True)
        for source in sorted(notices):
            shutil.copyfile(source, directory / source.name)
    (destination / "ANDROID-DEPENDENCIES.txt").write_text("\n".join(index) + "\n")
    sysroot = Path(run("rustc", "--print", "sysroot", capture=True).strip())
    rust_docs = sysroot / "share/doc/rust"
    toolchain = destination / "rust-toolchain"
    shutil.copytree(rust_docs / "licenses", toolchain)
    shutil.copyfile(rust_docs / "COPYRIGHT-library.html", toolchain / "COPYRIGHT-library.html")


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
        collect_notices(notices)
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
