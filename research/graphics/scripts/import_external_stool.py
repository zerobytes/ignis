#!/usr/bin/env python3
"""Pinned geometry-only glTF fixture. Powered by Poly Haven (Wooden Stool 02, CC0)."""

import hashlib, json, struct, subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "artifacts/appearance-probe/external-recovery"
FILES = {
    "stool.gltf": (
        "https://dl.polyhaven.org/file/ph-assets/Models/gltf/1k/wooden_stool_02/wooden_stool_02_1k.gltf",
        "44db244a27f82a7a8536c689e2995e79",
    ),
    "wooden_stool_02.bin": (
        "https://dl.polyhaven.org/file/ph-assets/Models/gltf/8k/wooden_stool_02/wooden_stool_02.bin",
        "ea27eb52e71c40319d9ca108843b19a0",
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
    g = json.loads((OUT / "source/stool.gltf").read_text())
    b = (OUT / "source/wooden_stool_02.bin").read_bytes()
    assert g["nodes"] == [{"mesh": 0, "name": "wooden_stool_02"}] and len(g["meshes"]) == 1

    def read(i, fmt):
        a = g["accessors"][i]
        v = g["bufferViews"][a["bufferView"]]
        assert not a.get("sparse") and not v.get("byteStride") and v["buffer"] == 0
        offset = a.get("byteOffset", 0) + v.get("byteOffset", 0)
        size = struct.calcsize(fmt)
        return list(struct.iter_unpack(fmt, b[offset : offset + size * a["count"]]))

    positions, normals, indices = [], [], []
    primitives = g["meshes"][0]["primitives"]
    assert len(primitives) == 2
    for prim in primitives:
        assert prim.get("mode", 4) == 4 and prim["material"] == 0
        base = len(positions)
        for key in ["POSITION", "NORMAL"]:
            accessor = g["accessors"][prim["attributes"][key]]
            assert accessor["type"] == "VEC3" and accessor["componentType"] == 5126
        positions.extend(read(prim["attributes"]["POSITION"], "<3f"))
        normals.extend(read(prim["attributes"]["NORMAL"], "<3f"))
        assert g["accessors"][prim["indices"]]["componentType"] == 5123
        indices.extend(base + v[0] for v in read(prim["indices"], "<H"))
    assert len(positions) == len(normals) and len(indices) % 3 == 0
    assert min(indices) >= 0 and max(indices) < len(positions)
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
    (OUT / "stool.igm").write_bytes(
        b"IGM1" + struct.pack("<IQQ", 1, len(payload), h) + payload
    )
    metadata = {
        "source": "https://polyhaven.com/a/wooden_stool_02",
        "author": "Kuutti Siitonen",
        "license": "CC0",
        "credit": "Powered by Poly Haven",
        "source_sha256": {
            name: hashlib.sha256((OUT / "source" / name).read_bytes()).hexdigest()
            for name in FILES
        },
        "gltf_vertices": len(positions),
        "primitives":len(primitives),
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
