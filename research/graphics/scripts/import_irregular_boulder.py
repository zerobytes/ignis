#!/usr/bin/env python3
"""Pinned geometry-only glTF fixture. Powered by Poly Haven (Boulder 01, CC0)."""

import hashlib, json, struct, subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "artifacts/appearance-probe/irregular-lod"
FILES = {
    "boulder.gltf": (
        "https://dl.polyhaven.org/file/ph-assets/Models/gltf/1k/boulder_01/boulder_01_1k.gltf",
        "26e2eef4a1f68c9557c65cd375b21d6c",
    ),
    "boulder_01.bin": (
        "https://dl.polyhaven.org/file/ph-assets/Models/gltf/8k/boulder_01/boulder_01.bin",
        "27a19fa031d8b4c3a09261f19c444dc5",
    ),
}


def main():
    (OUT / "source").mkdir(parents=True, exist_ok=True)
    for name, (url, md5) in FILES.items():
        path = OUT / "source" / name
        if not path.exists():
            subprocess.run(
                ["curl", "-fsSL", "--retry", "2", url, "-o", str(path)], check=True
            )
        assert hashlib.md5(path.read_bytes()).hexdigest() == md5, (
            name,
            "unexpected asset revision",
        )
    g = json.loads((OUT / "source/boulder.gltf").read_text())
    b = (OUT / "source/boulder_01.bin").read_bytes()
    assert g["nodes"] == [{"mesh": 0, "name": "boulder_01"}] and len(g["meshes"]) == 1

    def read(i, fmt):
        a = g["accessors"][i]
        v = g["bufferViews"][a["bufferView"]]
        assert not a.get("sparse") and not v.get("byteStride") and v["buffer"] == 0
        offset = a.get("byteOffset", 0) + v.get("byteOffset", 0)
        size = struct.calcsize(fmt)
        return list(struct.iter_unpack(fmt, b[offset : offset + size * a["count"]]))

    prim = g["meshes"][0]["primitives"][0]
    assert prim.get("mode", 4) == 4
    positions = read(prim["attributes"]["POSITION"], "<3f")
    normals = read(prim["attributes"]["NORMAL"], "<3f")
    indices = [v[0] for v in read(prim["indices"], "<I")]
    center = [
        (min(v[i] for v in positions) + max(v[i] for v in positions)) / 2
        for i in range(3)
    ]
    scale = 2 / max(
        max(v[i] for v in positions) - min(v[i] for v in positions) for i in range(3)
    )
    vertices = []
    lookup = {}
    remap = []
    for p, n in zip(positions, normals):
        p = [(p[i] - center[i]) * scale for i in range(3)]
        p = [p[0], -p[2], p[1]]
        n = [n[0], -n[2], n[1]]
        raw = struct.pack("<6f", *(0.0 if x == 0.0 else x for x in p + n))
        if raw not in lookup:
            lookup[raw] = len(vertices)
            vertices.append(raw)
        remap.append(lookup[raw])
    faces = [
        struct.pack("<4I", *(remap[i] for i in indices[j : j + 3]), 0)
        for j in range(0, len(indices), 3)
    ]
    payload = (
        struct.pack("<2I", len(vertices), len(faces))
        + b"".join(vertices)
        + b"".join(faces)
    )
    h = 0xCBF29CE484222325
    for v in payload:
        h = ((h ^ v) * 0x100000001B3) & ((1 << 64) - 1)
    (OUT / "boulder.igm").write_bytes(
        b"IGM1" + struct.pack("<IQQ", 1, len(payload), h) + payload
    )
    metadata = {
        "source": "https://polyhaven.com/a/boulder_01",
        "author": "Rico Cilliers",
        "license": "CC0",
        "credit": "Powered by Poly Haven",
        "source_sha256": {
            name: hashlib.sha256((OUT / "source" / name).read_bytes()).hexdigest()
            for name in FILES
        },
        "gltf_vertices": len(positions),
        "vertices": len(vertices),
        "triangles": len(faces),
        "center": center,
        "scale": scale,
        "axis": "glTF Y up to Z up: x,-z,y",
        "material": "shared diagnostic shader; original UV/textures not tested",
    }
    (OUT / "import.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(json.dumps(metadata))


if __name__ == "__main__":
    main()
