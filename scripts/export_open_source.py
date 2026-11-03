#!/usr/bin/env python3
"""Historical snapshot exporter; normal development uses the public Git origin.

Export one committed source snapshot without private history or local files.

This never creates a GitHub repository, pushes refs or changes the source tree.
The destination is a disposable, stable directory under this checkout's Git
metadata. Review its manifest and run a secret scanner before publication.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import tarfile


ROOT = Path(__file__).resolve().parents[1]
DIRECTORIES = {"src", "tests", "scripts", "examples", "assets", "docs", "website", ".cargo", ".github"}
FILES = {".gitignore", "Cargo.toml", "Cargo.lock", "build.rs", "README.md", "README.pt-BR.md",
         "LICENSE", "NOTICE", "THIRD_PARTY_NOTICES.md", "CONTRIBUTING.md", "CODE_OF_CONDUCT.md", "SECURITY.md"}
WITHHELD = ("docs/references/image-to-3d/", "docs/publication-internal.md")


def included(name):
    p = PurePosixPath(name)
    if p.is_absolute() or ".." in p.parts:
        raise ValueError("unsafe archive path")
    return (name in FILES or p.parts[0] in DIRECTORIES) and not any(
        name == prefix or name.startswith(prefix) for prefix in WITHHELD)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ref", default="HEAD", help="Committed revision to export (default: HEAD)")
    args = parser.parse_args()
    revision = subprocess.check_output(
        ["git", "rev-parse", "--verify", args.ref + "^{commit}"], cwd=ROOT, text=True).strip()
    git_dir = Path(subprocess.check_output(
        ["git", "rev-parse", "--absolute-git-dir"], cwd=ROOT, text=True).strip())
    destination = git_dir / "opensource-export"
    # Fixed generated directory only; never accept an arbitrary removal target.
    if destination.is_symlink():
        raise ValueError("export destination must not be a symlink")
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir()
    entries, omitted = [], []
    process = subprocess.Popen(["git", "archive", "--format=tar", revision], cwd=ROOT, stdout=subprocess.PIPE)
    try:
        with tarfile.open(fileobj=process.stdout, mode="r|") as archive:
            for member in archive:
                if member.isdir():
                    continue
                if not included(member.name):
                    omitted.append(member.name)
                    continue
                if not member.isfile():
                    raise ValueError("symlinks and special files require explicit review: " + member.name)
                target = destination / member.name
                target.parent.mkdir(parents=True, exist_ok=True)
                digest = hashlib.sha256()
                with archive.extractfile(member) as source, target.open("wb") as output:
                    while chunk := source.read(1024 * 1024):
                        digest.update(chunk)
                        output.write(chunk)
                target.chmod(0o755 if member.mode & 0o111 else 0o644)
                entries.append({"path": member.name, "sha256": digest.hexdigest(), "bytes": member.size})
        if process.wait() != 0:
            raise RuntimeError("git archive failed")
        if not FILES.issubset({e["path"] for e in entries}):
            raise ValueError("required public root files missing")
    except BaseException:
        process.kill()
        process.wait()
        shutil.rmtree(destination)
        raise
    finally:
        process.stdout.close()
    manifest = {"source_revision": revision, "files": entries, "omitted": omitted}
    manifest_path = git_dir / "opensource-export-manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps({"destination": str(destination), "manifest": str(manifest_path),
                      "files": len(entries), "omitted": len(omitted),
                      "bytes": sum(e["bytes"] for e in entries)}))


if __name__ == "__main__":
    main()
