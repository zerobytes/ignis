#!/usr/bin/env python3
"""Generate and review a living giant sequoia using only the native Rust grammar."""
import argparse
import json
import time
from pathlib import Path
from verify_lab import Client, ROOT
from demo_groundcover import fit

OUT = ROOT / 'artifacts/sequoia'

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    base=json.loads((ROOT/'artifacts/living-plants/empty.scene.json').read_text())
    base['name']='Sequoia-gigante viva'
    base['meshes']['assets'][0]['geometry']['positions']=[[-4000,-.025,-4000],[4000,-.025,-4000],[-4000,-.025,4000],[4000,-.025,4000]]
    (OUT/'empty.scene.json').write_text(json.dumps(base))
    c=Client(args.binary.resolve(),log_path=OUT/'runtime.log',startup_args=('--scene','artifacts/sequoia/empty.scene.json'))
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='sequoia',version='1')))
        c.request('notifications/initialized',notification=True)
        definition=json.loads((ROOT/'examples/sequoia.recipe.json').read_text())
        c.call('botany',operation=dict(kind='create_living',definition=definition,born=0))
        c.call('botany',operation=dict(kind='save',prefix='sequoia',path='assets/botany/sequoia.ignis-botany'))
        stages=[]
        for years in [0,1,5,20,100,300,1000]:
            started=time.perf_counter()
            c.call('botany',operation=dict(kind='living',prefix='sequoia',action=dict(kind='advance',time=years*365)))
            advance_ms=round((time.perf_counter()-started)*1000,2)
            r=c.call('botany',operation=dict(kind='inspect'))['recipes'][0]
            points=[p for o in r['source']['organs'] for p in o['path']]
            h=r['source']['organs'][0]['path'][-1][1]
            c.call('set_light',light=dict(direction=[.5,-.72,-.48],ambient=.2,direct_intensity=1.25,
                environment=dict(sky=[.42,.60,.8],horizon=[.66,.72,.72],ground=[.16,.18,.1],axis=[0,1,0],intensity=.4),
                atmosphere=dict(density=0,reduce_details=False,color=[.5,.65,.8],sun_color=[1,.9,.75],sun_strength=0,
                    layers=dict(sky_color=[.52,.7,.88],mountain_color=[.2,.3,.4],detail_start=10000,detail_end=20000,influence=0)),
                shadow=dict(center=[0,h*.45,0],extent=max(h*.7,3),resolution=2048,bias=.00035)))
            lo=[min(p[i] for p in points)-max(.3,h*.02) for i in range(3)]
            hi=[max(p[i] for p in points)+max(.3,h*.02) for i in range(3)]
            fit(c,lo,hi,OUT/f'age-{years}.png',direction=(.7,.25,1),width=1600,height=1600)
            stages.append(dict(years=years,height_m=h,base_diameter_m=2*r['source']['organs'][0]['shape']['radius'],organs=len(r['source']['organs']),advance_ms=advance_ms))
        # Reversible editor preview must not alter the canonical thousand-year tree.
        saved=r['living']
        c.call('botany',operation=dict(kind='open',open=True,prefix='sequoia'))
        for name,years in [('young-a',5),('old',1000),('young-b',5)]:
            c.call('botany',operation=dict(kind='preview_living_age',prefix='sequoia',time=years*365))
            c.call('capture',path=f'artifacts/sequoia/preview-{name}.png',width=1000,height=1200)
            assert c.call('botany',operation=dict(kind='inspect'))['recipes'][0]['living']==saved
        assert (OUT/'preview-young-a.png').read_bytes()==(OUT/'preview-young-b.png').read_bytes()
        assert (OUT/'preview-young-a.png').read_bytes()!=(OUT/'preview-old.png').read_bytes()
        c.call('botany',operation=dict(kind='cancel_preview'))
        c.call('capture',path='artifacts/sequoia/hero.png',width=2000,height=2000)
        c.call('save_scene',path='artifacts/sequoia/sequoia.scene.json')
        c.call('capture_editor',path='artifacts/sequoia/editor.png',width=1700,height=1200)
        c.call('load_scene',path='artifacts/sequoia/sequoia.scene.json')
        c.call('capture',path='artifacts/sequoia/roundtrip.png',width=2000,height=2000)
        assert (OUT/'hero.png').read_bytes()==(OUT/'roundtrip.png').read_bytes()
        # Persistent pruning is checked on a surviving old-age site, then restore.
        c.call('botany',operation=dict(kind='living',prefix='sequoia',action=dict(kind='prune',site='node8')))
        pruned=c.call('botany',operation=dict(kind='inspect'))['recipes'][0]
        assert not any(o['id']=='node8' or o['id'].startswith('node8/') for o in pruned['source']['organs'])
        c.call('save_scene',path='artifacts/sequoia/pruned.scene.json')
        c.call('load_scene',path='artifacts/sequoia/pruned.scene.json')
        assert c.call('botany',operation=dict(kind='inspect'))['recipes'][0]['living']==pruned['living']
        c.call('load_scene',path='artifacts/sequoia/sequoia.scene.json')
        c.call('botany',operation=dict(kind='living',prefix='sequoia',action=dict(kind='advance',time=3000*365)))
        c.call('load_scene',path='artifacts/sequoia/sequoia.scene.json')
        # Separate close view shows native bark, without changing the saved overview.
        c.call('set_camera',camera=dict(position=[15,7,20],target=[0,7,0],fov_degrees=44))
        c.call('capture',path='artifacts/sequoia/trunk.png',width=1600,height=1200)
        result=dict(stages=stages,preview_back_and_forth_png_exact=True,canonical_unchanged=True,roundtrip_png_exact=True,persistent_pruning=True,build_id=c.call('inspect')['build_id'])
        (OUT/'verification.json').write_text(json.dumps(result,indent=2)+'\n')
        cards=''.join(f'<figure><a href="age-{stage["years"]}.png"><img src="age-{stage["years"]}.png"></a><figcaption>{stage["years"]} anos · {stage["height_m"]:.2f}m · base Ø{stage["base_diameter_m"]:.2f}m</figcaption></figure>' for stage in stages)
        (OUT/'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Sequoia-gigante viva</title><style>body{background:#182029;color:#edf1e8;font:16px system-ui;max-width:1250px;margin:40px auto;padding:24px}img{width:100%}.hero{max-width:850px}.grid{display:grid;grid-template-columns:repeat(3,1fr);gap:18px}figure{margin:0}figcaption{padding:10px}a{color:#d3e5a8}</style><h1>Sequoia-gigante procedural</h1><p>Exemplar nativo de 1000 anos. Perfil autoral de Sequoiadendron giganteum; folhagem representada por agregados geométricos de raminhos. Cada imagem de idade tem enquadramento próprio.</p><img class="hero" src="hero.png"><h2>Desenvolvimento</h2><div class="grid">'+cards+'</div><h2>Casca procedural</h2><img src="trunk.png"><p>Verificações: prévia reversível 5 → 1000 → 5, estado canônico preservado, save/reopen PNG exato e poda persistente. Tempos de geração em <a href="verification.json">verification.json</a>; não são FPS.</p>')
        print(json.dumps(result))
    finally:c.close()

if __name__=='__main__':main()
