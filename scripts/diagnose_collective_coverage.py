#!/usr/bin/env python3
"""Diagnostic only: compare projected ICB1 source coverage with native PNGs.

No dependencies. This CPU projection is an independent approximate oracle, not
an implementation-specific guarantee about GPU precision or a quality certificate.
"""
import argparse
import json
import math
from pathlib import Path
import struct
import zlib


def mask_png(path, pixels=None):
    data = Path(path).read_bytes()
    at, compressed = 8, bytearray()
    while at < len(data):
        n = struct.unpack_from('>I', data, at)[0]
        kind, payload = data[at + 4:at + 8], data[at + 8:at + 8 + n]
        if kind == b'IHDR':
            w, h, bits, color, *_ = struct.unpack('>IIBBBBB', payload)
            assert bits == 8 and color == 6
        if kind == b'IDAT':
            compressed.extend(payload)
        at += n + 12
    raw = zlib.decompress(compressed)
    stride, previous, result = w * 4, bytearray(w * 4), set()
    at = 0
    for y in range(h):
        mode = raw[at]
        row = bytearray(raw[at + 1:at + 1 + stride])
        at += stride + 1
        for x in range(stride):
            left = row[x - 4] if x >= 4 else 0
            up, corner = previous[x], previous[x - 4] if x >= 4 else 0
            if mode == 1:
                predictor = left
            elif mode == 2:
                predictor = up
            elif mode == 3:
                predictor = (left + up) // 2
            elif mode == 4:
                p = left + up - corner
                distances = abs(p - left), abs(p - up), abs(p - corner)
                predictor = (left, up, corner)[distances.index(min(distances))]
            else:
                assert mode == 0
                predictor = 0
            row[x] = (row[x] + predictor) & 255
        result.update((x, y) for x in range(w) if min(row[x * 4:x * 4 + 3]) >= 250)
        if pixels is not None:
            pixels.extend(tuple(row[x*4:x*4+3]) for x in range(w))
        previous = row
    return w, h, result


def triangles(path):
    data = Path(path).read_bytes()
    assert data[:8] == b'IGCB1\0\0\0'
    nodes, count, parts = struct.unpack_from('<III', data, 8)
    at = 68
    for _ in range(parts):
        length = struct.unpack_from('<I', data, at)[0]
        at += 4 + length
    at += nodes * 32
    return [struct.unpack_from('<18fII', data, at + n * 80)[:9] for n in range(count)]


def unit(v):
    length = math.sqrt(sum(x * x for x in v))
    return tuple(x / length for x in v)


def cross(a, b):
    return a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]


def edge(a, b, p):
    return (b[0]-a[0])*(p[1]-a[1])-(b[1]-a[1])*(p[0]-a[0])


def projected_geometry(source, camera, scale, w, h):
    eye, target = camera['position'], camera['target']
    forward = unit(tuple(t-e for t, e in zip(target, eye)))
    right = unit(cross(forward, (0, 1, 0)))
    up = cross(right, forward)
    focal = h / (2 * math.tan(math.radians(camera['fov_degrees']) / 2))
    result = []
    for triangle in source:
        points = []
        for j in range(0, 9, 3):
            p = tuple(triangle[j+k] * scale + (50 if k == 1 else 0) - eye[k] for k in range(3))
            depth = sum(a*b for a, b in zip(p, forward))
            if depth <= 0.05:
                break
            points.append((w/2 + focal * sum(a*b for a, b in zip(p, right)) / depth,
                           h/2 - focal * sum(a*b for a, b in zip(p, up)) / depth))
        if len(points) == 3:
            result.append(points)
    return result


def raster(points, bits, w, h):
    mask = set()
    for triangle in points:
        if bits:
            factor = 2**bits
            triangle = [tuple(round(x * factor) / factor for x in p) for p in triangle]
        a, b, c = triangle
        area = edge(a, b, c)
        if not area:
            continue
        if area < 0:
            b, c = c, b
        xmin = max(0, math.ceil(min(a[0], b[0], c[0]) - .5))
        xmax = min(w - 1, math.floor(max(a[0], b[0], c[0]) - .5))
        ymin = max(0, math.ceil(min(a[1], b[1], c[1]) - .5))
        ymax = min(h - 1, math.floor(max(a[1], b[1], c[1]) - .5))
        edges = [(a,b), (b,c), (c,a)]
        included = [end[1] < start[1] or (end[1] == start[1] and end[0] > start[0]) for start,end in edges]
        for y in range(ymin, ymax+1):
            for x in range(xmin, xmax+1):
                values = [edge(start, end, (x+.5, y+.5)) for start,end in edges]
                if all(v > 0 or (v == 0 and top) for v, top in zip(values,included)):
                    mask.add((x,y))
    return mask


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scene', default='artifacts/expanded-world/world.scene.json')
    parser.add_argument('--packet', default='artifacts/collective-lod/object.igcb')
    parser.add_argument('--captures', default='artifacts/collective-lod/final-isolated')
    parser.add_argument('--output', default='artifacts/collective-lod/coverage-probe/projection.json')
    args = parser.parse_args()
    scene = json.loads(Path(args.scene).read_text())
    member = scene['meshes']['objects'][0]['members'][0]
    scale = next(i['scale'] for i in scene['meshes']['instances'] if i['id'] == member)
    source = triangles(args.packet)
    captures = Path(args.captures)
    rows = []
    for row in json.loads((captures/'isolated.json').read_text()):
        pose = row['pose']
        w, h, reference = mask_png(captures/f'coverage-{pose}-source.png')
        _, _, query = mask_png(captures/f'coverage-{pose}-collective.png')
        points = projected_geometry(source, row['camera'], scale, w, h)
        results = {}
        for bits in [0,4,6,8,10,12]:
            mask = raster(points,bits,w,h)
            results[str(bits)] = {'pixels':len(mask),'native_difference':len(mask^reference), 'query_difference':len(mask^query)}
        rows.append({'pose':pose,'native_pixels':len(reference),'query_difference':len(reference^query),'cpu_projection':results})
    Path(args.output).parent.mkdir(parents=True,exist_ok=True)
    Path(args.output).write_text(json.dumps(rows,indent=2)+'\n')
    print(json.dumps(rows,indent=2))


if __name__ == '__main__':
    main()
