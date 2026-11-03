#!/usr/bin/env python3
"""Native PNG comparison through MCP, with explicit support and state invariance."""
import argparse
import json
import struct
import zlib
from pathlib import Path
from verify_lab import Client, ROOT


def write_rgb(path, width, height, pixels):
    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
    rows = b''.join(b'\0' + bytes(sum(pixels[y*width:(y+1)*width], [])) for y in range(height))
    path.write_bytes(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
                     + chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND', b''))


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    call = c.call
    try:
        c.request('initialize', {'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'image-similarity','version':'1'}})
        c.request('notifications/initialized', notification=True)
        assert 'compare_images' in json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema'])
        call('generate', recipe={'seed':7,'width':16,'depth':14,'obstacles':3})
        call('step', ticks=7)
        call('capture', path='artifacts/sim-native-a.png', width=128, height=96)
        call('capture', path='artifacts/sim-native-b.png', width=128, height=96)
        before = call('inspect')
        request = dict(reference_path='artifacts/sim-native-a.png', candidate_path='artifacts/sim-native-b.png',
                       difference_path='artifacts/sim-native-difference.png')
        same = call('compare_images', **request)
        assert same == call('compare_images', **request)
        m = same['report']['metrics']
        assert m['exact_rgb_match'] and m['rgb_mae'] == m['rgb_rmse'] == 0
        assert abs(m['ssim_luma'] - 1) < 1e-10 and m['ssim_windows'] == (128-10)*(96-10)
        assert call('inspect') == before and (ROOT/'artifacts/sim-native-difference.png').exists()
        checks.append('Same native render compares exactly twice; SSIM support and definitions explicit, difference file and zero state/history/tick changes')

        call('save_replay', path='artifacts/sim-native.replay.json')
        call('capture_replay', replay_path='artifacts/sim-native.replay.json', path='artifacts/sim-native-replay.png', width=128, height=96)
        assert call('compare_images', reference_path=request['reference_path'], candidate_path='artifacts/sim-native-replay.png')['report']['metrics']['exact_rgb_match']
        camera = before['scene']['camera']; changed = {**camera, 'position':[camera['position'][0]+2, *camera['position'][1:]]}
        call('set_camera', camera=changed)
        call('capture', path='artifacts/sim-native-shifted.png', width=128, height=96)
        shifted = call('compare_images', reference_path=request['reference_path'], candidate_path='artifacts/sim-native-shifted.png')['report']['metrics']
        assert not shifted['exact_rgb_match'] and shifted['rgb_mae'] > 0 and shifted['ssim_luma'] < .99
        checks.append('Capture/replay pixels agree through comparator; explicitly changed native camera produces spatial error, without implicit alignment')

        w, h = 128, 96
        a = [[255]*3 for _ in range(w*h)]; b = [p[:] for p in a]; mask = [p[:] for p in a]
        for y in range(40,56):
            for x in range(48,64):
                b[y*w+x] = [0]*3; mask[y*w+x] = [0]*3
        for name, pixels in [('reference',a),('defect',b),('mask',mask)]:
            write_rgb(ROOT/f'artifacts/sim-{name}.png',w,h,pixels)
        pair = dict(reference_path='artifacts/sim-reference.png',candidate_path='artifacts/sim-defect.png')
        roi = dict(id='door', x=48,y=40,width=16,height=16)
        report = call('compare_images', **pair, regions=[roi])['report']
        assert report['metrics']['rgb_mae'] < .021
        assert report['regions'][0]['metrics']['rgb_mae'] == 1
        assert abs(report['regions'][0]['metrics']['ssim_luma'] - .0001/1.0001) < 1e-10
        assert report['regions'][0]['metrics']['ssim_windows'] == 36
        masked = call('compare_images', **pair, mask_path='artifacts/sim-mask.png')['report']
        assert masked['metrics']['exact_rgb_match'] and masked['metrics']['selected_pixels'] == w*h-256
        checks.append('Dominant equal background hides local defect globally; named region reveals full error and analytic SSIM, mask reports its reduced coverage')
        sparse = [[0]*3 for _ in a]; sparse[0] = [255]*3
        write_rgb(ROOT/'artifacts/sim-sparse.png',w,h,sparse)
        sparse_report = call('compare_images', **pair, mask_path='artifacts/sim-sparse.png')['report']
        assert sparse_report['metrics']['ssim_luma'] is None and sparse_report['metrics']['selected_pixels'] == 1
        write_rgb(ROOT/'artifacts/sim-empty.png',w,h,[[0]*3 for _ in a])
        write_rgb(ROOT/'artifacts/sim-nonbinary.png',w,h,[[128]*3 for _ in a])
        write_rgb(ROOT/'artifacts/sim-small.png',11,11,[[0]*3 for _ in range(121)])
        before = call('inspect'); original = (ROOT/pair['reference_path']).read_bytes()
        failures = [dict(mask_path='artifacts/sim-empty.png'), dict(mask_path='artifacts/sim-nonbinary.png'),
                    dict(mask_path='artifacts/sim-mask.png', regions=[roi]), dict(regions=[{**roi,'width':129}]),
                    dict(regions=[roi,roi]), dict(candidate_path='artifacts/sim-small.png'),
                    dict(reference_path='../out.png'), dict(difference_path=pair['reference_path']),
                    dict(candidate_path='artifacts/missing.png',difference_path='artifacts/sim-new/output.png')]
        for options in failures:
            call('compare_images', **{**pair, **options}, expected_error=True)
            assert call('inspect') == before
        call('compare_images', **pair, expected_revision=99999, expected_error=True)
        assert (ROOT/pair['reference_path']).read_bytes() == original
        assert not (ROOT/'artifacts/sim-new').exists()
        checks.append('Sparse selection returns absent SSIM; empty/nonbinary masks, empty ROI, invalid dimensions/regions, traversal, input overwrite and stale revision reject atomically')
        data = {'passed':True,'checks':len(checks),'details':checks,'build_id':before['build_id'],
                'native_window':window,'native_same':same['report'],'native_camera_shift':shifted,
                'background_defect':report,'masked_defect':masked}
        (ROOT/'artifacts/image-similarity-verification.json').write_text(json.dumps(data,indent=2)+'\n')
        print(json.dumps(data,indent=2))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--window',action='store_true')
    args = parser.parse_args()
    verify(args.binary,args.window)
