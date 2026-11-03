#!/usr/bin/env python3
"""Resolve Instruments XML references and measure unions of hardware intervals.

GPU vertex/fragment channels overlap: durations must not simply be summed.
Only profile-runner intervals with completed command buffers are accepted.
"""
import collections
import json
import statistics
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'artifacts/botanical-valley/performance'


class Table:
    def __init__(self, path):
        self.root = ET.parse(path).getroot()
        self.ids = {n.attrib['id']: n for n in self.root.iter() if 'id' in n.attrib}
        self.node = self.root.find('node')
        self.columns = [c.findtext('mnemonic') for c in self.node.find('schema')]

    def resolve(self, node):
        while 'ref' in node.attrib:
            node = self.ids[node.attrib['ref']]
        return node

    def text(self, node):
        return self.resolve(node).text or ''

    def fmt(self, node):
        return self.resolve(node).attrib.get('fmt', '')

    def rows(self):
        for row in self.node.findall('row'):
            yield dict(zip(self.columns, row))


def union(intervals):
    total, end = 0, -1
    for start, stop in sorted(intervals):
        total += max(0, stop - max(end, start))
        end = max(end, stop)
    return total


def main():
    completed = Table(OUT / 'gpu-full-buffers.xml')
    buffers = {completed.text(r['cmdbuffer-id']) for r in completed.rows()}
    gpu = Table(OUT / 'gpu-full-intervals.xml')
    groups = collections.defaultdict(list)
    for row in gpu.rows():
        if 'profile-runner' not in gpu.fmt(row['process']):
            continue
        label = gpu.fmt(row['event-label'])
        kind = 'shadow' if label.startswith('directional shadow:') else 'scene' if 'scene' in label else 'other'
        if kind == 'other':
            continue
        start, duration = int(gpu.text(row['start'])), int(gpu.text(row['duration']))
        groups[(int(gpu.text(row['frame-number'])), kind)].append(dict(
            start=start, end=start + duration, buffer=gpu.text(row['cmdbuffer-id']), channel=gpu.text(row['channel-name'])))
    passes = []
    for (number, kind), rows in sorted(groups.items()):
        if any(row['buffer'] not in buffers for row in rows):
            continue
        intervals = [(r['start'], r['end']) for r in rows]
        passes.append(dict(trace_frame=number, kind=kind,
            start_ns=min(r['start'] for r in rows), end_ns=max(r['end'] for r in rows),
            active_ms=union(intervals) / 1e6,
            span_ms=(max(r['end'] for r in rows) - min(r['start'] for r in rows)) / 1e6,
            vertex_active_ms=union([(r['start'], r['end']) for r in rows if r['channel'] == 'Vertex']) / 1e6,
            intervals=intervals, buffer_completion_verified=True))
    pairs = []
    for shadow in [p for p in passes if p['kind'] == 'shadow']:
        scene = next((p for p in passes if p['kind'] == 'scene' and p['start_ns'] >= shadow['start_ns']), None)
        if scene is None:
            continue
        pairs.append(dict(shadow_trace_frame=shadow['trace_frame'], scene_trace_frame=scene['trace_frame'],
            gpu_active_ms=union(shadow['intervals'] + scene['intervals']) / 1e6,
            gpu_span_ms=(scene['end_ns'] - shadow['start_ns']) / 1e6))
    assert len(pairs) >= 3, 'Insufficient complete GPU pass pairs'
    for p in passes:
        del p['intervals']
    summary = {}
    for kind in ('shadow', 'scene'):
        rows = [p for p in passes if p['kind'] == kind]
        summary[kind] = dict(samples=len(rows), median_active_ms=statistics.median(p['active_ms'] for p in rows),
            median_span_ms=statistics.median(p['span_ms'] for p in rows),
            median_vertex_active_ms=statistics.median(p['vertex_active_ms'] for p in rows))
    summary['combined'] = dict(samples=len(pairs), median_active_ms=statistics.median(p['gpu_active_ms'] for p in pairs),
        median_span_ms=statistics.median(p['gpu_span_ms'] for p in pairs))
    cpu = Table(OUT / 'cpu-profile.xml')
    categories = collections.Counter()
    for row in cpu.rows():
        if 'profile-runner' not in cpu.fmt(row['process']):
            continue
        names = [cpu.resolve(f).attrib.get('name', '') for f in cpu.resolve(row['stack']).findall('frame')]
        if not any('5ignis6render' in n and 'Renderer4draw' in n for n in names):
            continue
        if any('command_encoder_finish' in n for n in names):
            category = 'wgpu_finish_and_encode_render_passes'
        elif any('render_visibility' in n and 'Visibility' in n and 'update' in n for n in names):
            category = 'visibility_update'
        elif any('render_instances' in n and '3Gpu4draw' in n for n in names):
            category = 'record_instanced_draw_commands'
        else:
            category = 'other_renderer_prepare_and_submit'
        categories[category] += int(cpu.text(row['weight']))
    total = sum(categories.values())
    cpu_summary = {k: dict(sampled_cpu_ms=v / 1e6, percent_of_renderer_draw_samples=100 * v / total) for k, v in categories.items()}
    report = dict(gpu=summary, passes=passes, pairs=pairs, cpu_sampled_categories=cpu_summary,
        method='Metal hardware Active intervals filtered to profile-runner, verified completed command buffers. Union overlapping vertex/fragment/nesting intervals, group labelled shadow/scene passes, pair in chronological order. GPU span includes scheduling gaps; active union excludes gaps. CPU stack categories are disjoint statistical samples during tracing, not exact clocks or percentages of uninstrumented frame wall time. CPU/GPU intervals can overlap; do not add all measurements as independent wall stages.')
    (OUT / 'gpu-cpu-trace-summary.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(dict(gpu=summary, cpu_sampled_categories=cpu_summary), indent=2))


if __name__ == '__main__':
    main()
