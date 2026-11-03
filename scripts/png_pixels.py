"""Decode bounded native RGBA8 capture pixels for numerical GPU comparisons."""
import struct
import zlib


def rgba8(data):
    assert data[:8] == b'\x89PNG\r\n\x1a\n'
    offset = 8
    compressed = bytearray()
    width = height = None
    while offset < len(data):
        length = struct.unpack_from('>I', data, offset)[0]
        kind = data[offset+4:offset+8]
        body = data[offset+8:offset+8+length]
        if kind == b'IHDR':
            width, height, depth, color, compression, filtering, interlace = struct.unpack('>IIBBBBB', body)
            assert 1 <= width <= 2048 and 1 <= height <= 2048
            assert (depth, color, compression, filtering, interlace) == (8, 6, 0, 0, 0)
        elif kind == b'IDAT':
            compressed.extend(body)
        elif kind == b'IEND':
            break
        offset += 12 + length
    assert width is not None and height is not None
    stride = width * 4
    raw = zlib.decompress(compressed)
    assert len(raw) == height * (stride + 1)
    out = bytearray()
    previous = bytearray(stride)
    for y in range(height):
        start = y * (stride + 1)
        mode = raw[start]
        assert 0 <= mode <= 4
        row = bytearray(raw[start+1:start+1+stride])
        for x in range(stride):
            left = row[x-4] if x >= 4 else 0
            up = previous[x]
            corner = previous[x-4] if x >= 4 else 0
            if mode == 1: prediction = left
            elif mode == 2: prediction = up
            elif mode == 3: prediction = (left+up)//2
            elif mode == 4:
                p = left+up-corner
                distances = [abs(p-left), abs(p-up), abs(p-corner)]
                prediction = [left, up, corner][distances.index(min(distances))]
            else: prediction = 0
            row[x] = (row[x]+prediction) & 255
        out.extend(row)
        previous = row
    return width, height, bytes(out)


def max_error(a, b):
    wa, ha, pa = rgba8(a)
    wb, hb, pb = rgba8(b)
    assert (wa, ha) == (wb, hb)
    return max(abs(x-y) for x, y in zip(pa, pb))
