#!/usr/bin/env python3
"""Package the current local build without duplicating the binary's contents."""
import argparse
import errno
import os
from pathlib import Path
import plistlib
import shlex


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--editor", action="store_true", help="Open the native editor")
    parser.add_argument("--install", action="store_true", help="Use ~/Applications directly")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    binary = (root / "target/release/ignis").resolve()
    if not binary.is_file():
        parser.error("Run cargo build --locked --release first")
    name = "Ignis Editor" if args.editor else "Ignis"
    identifier = "dev.ignisengine.editor" if args.editor else "dev.ignisengine.laboratory"
    destination = Path.home() / "Applications" if args.install else root / "artifacts"
    app = destination / f"{name}.app"
    contents = app / "Contents"
    macos = contents / "MacOS"
    info = contents / "Info.plist"
    if app.exists():
        if app.is_symlink() or not info.is_file():
            parser.error(f"Refusing to replace an unrecognized app: {app}")
        with info.open("rb") as file:
            if plistlib.load(file).get("CFBundleIdentifier") != identifier:
                parser.error(f"Another application already occupies {app}")
    macos.mkdir(parents=True, exist_ok=True)
    temporary = macos / ".ignis-link"
    temporary.unlink(missing_ok=True)
    try:
        os.link(binary, temporary)
    except OSError as error:
        if error.errno != errno.EXDEV:
            raise
        # A local worktree may keep its current build on another mounted volume.
        temporary.symlink_to(binary)
    os.replace(temporary, macos / "ignis")
    options = ["--root", str(root)]
    if args.editor:
        options += ["--editor"]
    launcher = macos / "launch"
    temporary = macos / ".launch"
    temporary.write_text(
        '#!/bin/sh\nexec "$(dirname "$0")/ignis" '
        + shlex.join(options) + ' "$@"\n', encoding="utf-8"
    )
    temporary.chmod(0o755)
    os.replace(temporary, launcher)
    temporary = contents / ".Info.plist"
    with temporary.open("wb") as file:
        plistlib.dump({
            "CFBundleIdentifier": identifier,
            "CFBundleName": name,
            "CFBundleDisplayName": name,
            "CFBundleExecutable": "launch",
            "CFBundlePackageType": "APPL",
            "CFBundleShortVersionString": "0.1.0",
            "CFBundleVersion": "1",
            "NSHighResolutionCapable": True,
        }, file)
    os.replace(temporary, info)
    (macos / "laboratory").unlink(missing_ok=True)
    print(app)


if __name__ == "__main__":
    main()
