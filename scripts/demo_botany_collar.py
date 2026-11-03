#!/usr/bin/env python3
"""Before/after native crown review using the same root curves and cameras."""
import copy,json,subprocess,sys
import demo_botany_buttress as demo
from verify_lab import ROOT

if __name__=='__main__':
    demo.OUT=ROOT/'artifacts/botany-collar'
    demo.BASELINE='2d49df8'
    base=json.loads(subprocess.run(['git','show',f'{demo.BASELINE}:assets/botany/figueira_rs.ignis-botany'],cwd=ROOT,check=True,capture_output=True).stdout)
    demo.DEFAULT=base['source']['root_systems'][0]['settings']
    demo.DOC='botany-collar'
    demo.ANALYSIS='collar-correction-analysis'
    demo.COMPARE_VIEW='side'
    demo.COMPARE_REGIONS=[dict(id='collar',x=400,y=260,width=500,height=450),dict(id='ledge',x=530,y=390,width=330,height=270)]
    demo.PRESERVE_ALL_ORGANS=True
    demo.REFERENCE_IMAGES=('reference-collar.png','reference-ledge.png')
    if '--local' in sys.argv:
        sys.argv.remove('--local')
        if '--publish-source' in sys.argv or '--appearance-only' in sys.argv:
            raise ValueError('Local diagnostic cannot publish the canonical source')
        local=copy.deepcopy(base);stem=local['source']['root_systems'][0]['stem']
        local['settings'].update(leaf_density=0.,bark_color=[.43,.43,.43]);local['pruned']=[]
        local['source']['organs']=[o for o in local['source']['organs'] if (o['id']==stem or o.get('parent')==stem or '/root' in o['id']) and '/epifita' not in o['id']]
        demo.OUT.mkdir(parents=True,exist_ok=True)
        demo.SOURCE_OVERRIDE=demo.OUT/'local-before.ignis-botany'
        demo.SOURCE_OVERRIDE.write_text(json.dumps(local,separators=(',',':')))
        demo.OUT=demo.OUT/'local'
        demo.review=lambda report:None
    else:
        original_review=demo.review
        def review(report):
            for name in ['before','after']:
                subprocess.run(['sips','-c','450','500','--cropOffset','260','400',str(demo.OUT/f'{name}-side.png'),'--out',str(demo.OUT/f'detail-{name}.png')],check=True,capture_output=True)
            original_review(report)
            path=demo.OUT/'review.html';html=path.read_text()
            html=html.replace('<h1>Raízes em paredes e cristas</h1>','<h1>Correção do colo e da borda entre raízes</h1><h2>Detalhe: antes e depois</h2><div class="row"><img src="detail-before.png"><img src="detail-after.png"></div>')
            path.write_text(html)
        demo.review=review
    demo.main()
