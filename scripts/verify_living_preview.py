#!/usr/bin/env python3
# /// script
# dependencies = ["pillow>=11,<13"]
# ///
"""Verify reversible native age previews and untouched canonical persistence."""
import json
from verify_lab import Client, ROOT
from PIL import Image

OUT = ROOT/'artifacts/living-tree'

def main():
    c = Client(ROOT/'target/release/ignis', log_path=OUT/'preview.log',
        startup_args=('--scene', 'artifacts/living-tree/tree.scene.json'))
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='living-preview-proof', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('botany', operation=dict(kind='load', path='assets/botany/living-tree.ignis-botany', publish=True))
        c.call('botany', operation=dict(kind='living', prefix='living', action=dict(kind='advance', time=12*365)))
        c.call('botany', operation=dict(kind='open', open=True, prefix='living'))
        saved = c.call('botany', operation=dict(kind='inspect'))['recipes']
        def capture(name):
            c.call('capture', path=f'artifacts/living-tree/preview-{name}.png', width=1400, height=1100)
        capture('saved')
        c.call('capture_editor', path='artifacts/living-tree/preview-editor-saved.png', width=1600, height=1100)
        for years, name in [(5, 'young'), (20, 'adult'), (5, 'young-again')]:
            r = c.call('botany', operation=dict(kind='preview_living_age', prefix='living', time=years*365))
            assert r['world_changed'] is False
            assert c.call('botany', operation=dict(kind='inspect'))['recipes'] == saved
            capture(name)
            c.call('capture_editor', path=f'artifacts/living-tree/preview-editor-{name}.png', width=1600, height=1100)
        def viewport(name):
            with Image.open(OUT/f'preview-editor-{name}.png') as image:
                return image.crop((48, 100, 1200, 1050)).tobytes()
        assert viewport('young') == viewport('young-again')
        assert viewport('young') != viewport('adult')
        assert viewport('young') != viewport('saved')
        assert (OUT/'preview-young.png').read_bytes() == (OUT/'preview-young-again.png').read_bytes()
        assert (OUT/'preview-young.png').read_bytes() != (OUT/'preview-adult.png').read_bytes()
        c.call('capture_editor', path='artifacts/living-tree/preview-editor.png', width=1600, height=1100)
        c.call('save_scene', path='artifacts/living-tree/preview-saved.scene.json')
        c.call('botany', operation=dict(kind='cancel_preview'))
        capture('cancelled')
        c.call('capture_editor', path='artifacts/living-tree/preview-editor-cancelled.png', width=1600, height=1100)
        assert viewport('saved') == viewport('cancelled')
        assert (OUT/'preview-saved.png').read_bytes() == (OUT/'preview-cancelled.png').read_bytes()
        c.call('load_scene', path='artifacts/living-tree/preview-saved.scene.json')
        assert c.call('botany', operation=dict(kind='inspect'))['recipes'] == saved
        capture('reopened')
        assert (OUT/'preview-saved.png').read_bytes() == (OUT/'preview-reopened.png').read_bytes()
        report = dict(editor_viewport_back_and_forth_exact=True, editor_viewport_changes_with_age=True, editor_cancel_restores_exact=True, back_and_forth_png_exact=True, different_ages_differ=True,
            cancel_restores_png_exact=True, save_during_preview_canonical=True,
            reopen_png_exact=True, build_id=c.call('inspect')['build_id'])
        (OUT/'preview-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report))
    finally:
        c.close()

if __name__ == '__main__':
    main()
