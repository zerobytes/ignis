"""Summarize paired total costs; preserve all raw samples in report.json."""
import json
import statistics as st
from pathlib import Path

ROOT = Path('artifacts/appearance-probe/gpu-visibility')
METRICS = ['gpu_ms', 'serial_ms', 'moving_throughput_ms']


def summarize(data):
    result = []
    for resolution in range(2):
        for scenario in data['scenarios']:
            rows = [r for r in data['rows'] if r['resolution'] == resolution and r['scenario'] == scenario]
            timed = [r for r in rows if r['timings']]

            def pair(r, mode, key):
                return st.mean(r['timings'][i][key]['median'] for i in [mode, mode + 6])

            modes = []
            for mode, name in enumerate(data['modes']):
                modes.append({
                    'name': name,
                    'triangle_fraction_median': st.median(r['counts'][mode]['triangles'] / r['counts'][0]['triangles'] for r in rows),
                    'ms': {k: st.median(pair(r, mode, k) for r in timed) for k in METRICS},
                    'relative_full': {k: st.median(pair(r, mode, k) / pair(r, 0, k) for r in timed) for k in METRICS},
                    'duplicate_ratio_range': {k: [min(r['timings'][mode+6][k]['median'] / r['timings'][mode][k]['median'] for r in timed), max(r['timings'][mode+6][k]['median'] / r['timings'][mode][k]['median'] for r in timed)] for k in METRICS},
                })
            result.append({'resolution': resolution, 'scenario': scenario, 'modes': modes})
    return result


def main():
    data = json.loads((ROOT/'report.json').read_text())
    assert len(data['modes']) == 6
    data['summary'] = summarize(data)
    data['edges'] = json.loads((ROOT/'edge-checks.json').read_text())
    (ROOT/'summary.json').write_text(json.dumps(data['summary'], indent=2))
    # The interactive page needs medians only, not thousands of timing samples.
    for r in data['rows']:
        if r['timings']:
            for t in r['timings']:
                for k in t:
                    if isinstance(t[k], dict):
                        t[k] = {'median': t[k]['median']}
    template = Path(__file__).with_name('gpu_visibility_report.html').read_text()
    (ROOT/'index.html').write_text(template.replace('__DATA__', json.dumps(data, separators=(',', ':'))))
    print(json.dumps(data['summary'], indent=2))


if __name__ == '__main__':
    main()
