"""Collect the license notices for packaged Showcase builds."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def run(*args, capture=False, env=None):
    return subprocess.run(
        args, cwd=ROOT, check=True, text=True, capture_output=capture, env=env
    ).stdout


def collect_notices(destination, targets, feature):
    flags = ["--locked", "--features", feature]
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
    for target in targets:
        tree = run("cargo", "tree", *flags, "--target", target, "--edges", "normal,build", "--prefix",
                   "none", "--format", "{p}", capture=True)
        for line in tree.splitlines():
            match = re.match(r"^(\S+) v(\S+)", line)
            if match:
                linked.add(match.groups())
        metadata = json.loads(run("cargo", "metadata", *flags, "--format-version",
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
    (destination / "DEPENDENCIES.txt").write_text("\n".join(index) + "\n", encoding="utf-8")
    sysroot = Path(run("rustc", "--print", "sysroot", capture=True).strip())
    rust_docs = sysroot / "share/doc/rust"
    toolchain = destination / "rust-toolchain"
    shutil.copytree(rust_docs / "licenses", toolchain)
    shutil.copyfile(rust_docs / "COPYRIGHT-library.html", toolchain / "COPYRIGHT-library.html")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    parser.add_argument("target")
    args = parser.parse_args()
    args.destination.mkdir(parents=True, exist_ok=True)
    collect_notices(args.destination, [args.target], "demo")
