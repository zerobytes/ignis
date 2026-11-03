"""Extract native-pixel reference/render crops and assemble the E02 review report."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
from PIL import Image, ImageDraw

PROJECT=Path(__file__).resolve().parents[1]
ROOT=PROJECT/'artifacts/house-e02'
SOURCE=Path('/Volumes/Zero/EpicGames/Unreal Projects/LandOfChaos/docs/art/settlements/residential-concepts-v2/E02.png')
EXPECTED='4153a456f9fdf2e9e6f43328171692156db7563c2062654199e66c0c540882ad'
BOXES=[('mass',[65,15,1310,800]),('main-roof',[680,15,690,500]),
       ('left-roofs',[65,175,785,395]),('porch',[90,470,750,300]),
       ('entry',[575,475,260,260]),('great-window',[815,355,355,425]),
       ('side-windows',[1110,440,220,340]),('upper-gallery',[430,320,400,150]),
       ('courtyard',[150,685,980,345]),('rail-left',[130,710,440,260]),
       ('rail-right',[865,725,275,285]),('dormer',[1090,205,185,210]),
       ('plants',[1080,675,295,230]),('tree-detail',[900,425,175,230]),
       ('column-detail',[385,480,75,265]),('entry-tree',[695,505,85,155]),
       ('rail-tree',[355,805,65,105]),('floor-medallion',[565,735,225,100])]
PARENTS={'tree-detail':'great-window','column-detail':'porch','entry-tree':'entry',
         'rail-tree':'rail-left','floor-medallion':'courtyard'}


def prepare(reference=None, report=False):
    source=Path(reference) if reference else SOURCE
    if not source.exists():source=PROJECT/'docs/references/image-to-3d/e02.png'
    digest=hashlib.sha256(source.read_bytes()).hexdigest()
    if digest!=EXPECTED:raise ValueError('This evidence manifest belongs to the supplied E02 image')
    ROOT.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(source,ROOT/'reference.png')
    original=Image.open(source)
    if original.size!=(1402,1122):raise ValueError('Unexpected reference dimensions')
    candidates={}
    for name,file in [('candidate','render.png'),('baseline-crops','baseline.png')]:
        if (ROOT/file).exists():
            candidates[name]=Image.open(ROOT/file)
            if candidates[name].size!=original.size:raise ValueError('Candidate grid differs; no automatic resize')
    source_rows={}
    for line in (PROJECT/'docs/e02-reconstruction.md').read_text().splitlines():
        if line.startswith('| F'):
            cells=[c.strip() for c in line.split('|')[1:-1]]
            source_rows[cells[0].split()[0]]={'analysis':cells[1],'construction':cells[2]}
    review_path=PROJECT/'docs/references/image-to-3d/e02-review.json'
    reviews=json.loads(review_path.read_text()) if review_path.exists() else {}
    items=[]
    for i,(name,box) in enumerate(BOXES):
        x,y,w,h=box
        if min(x,y)<0 or min(w,h)<=0 or x+w>original.width or y+h>original.height:
            raise ValueError(f'Invalid crop: {name}')
        item={'id':name,'feature_id':f'F{i+1:02d}','box':box,'parent':PARENTS.get(name),
              'source_sha256':digest,'analysis':source_rows[f'F{i+1:02d}']['analysis'],
              'construction':source_rows[f'F{i+1:02d}']['construction'],
              'review':reviews.get(name,{'status':'pendente','diagnosis':'Revisão visual ainda não registrada.'})}
        for folder,im in [('reference',original),*candidates.items()]:
            (ROOT/folder).mkdir(exist_ok=True)
            path=ROOT/folder/f'{name}.png';im.crop((x,y,x+w,y+h)).save(path)
            item[folder]=f'{folder}/{name}.png'
        items.append(item)
    manifest={'source':str(source),'sha256':digest,'dimensions':list(original.size),
              'coordinates':'top-left; x right; y down; native pixels, no resize/alignment','regions':items}
    (ROOT/'evidence.json').write_text(json.dumps(manifest,indent=2,ensure_ascii=False)+'\n')
    for folder in ['reference',*candidates]:
        sheet=Image.new('RGB',(1400,1250),(225,222,215));draw=ImageDraw.Draw(sheet)
        for i,item in enumerate(items):
            im=Image.open(ROOT/item[folder]);im.thumbnail((335,218))
            x=(i%4)*350;y=(i//4)*250;sheet.paste(im,(x,y+26))
            draw.text((x+4,y+5),item['feature_id']+' '+item['id'],fill=(25,28,26))
        sheet.save(ROOT/f'{folder}-contact.png')
    if report:
        for filename in ['comparison.json','baseline-comparison.json','capture.json','verification.json','parameters.json','mapping.json']:
            manifest[filename.removesuffix('.json')]=json.loads((ROOT/filename).read_text())
        (ROOT/'report.json').write_text(json.dumps(manifest,indent=2,ensure_ascii=False)+'\n')
        template=(PROJECT/'scripts/e02_report.html').read_text()
        payload=json.dumps(manifest,ensure_ascii=False).replace('<','\\u003c')
        (ROOT/'index.html').write_text(template.replace('__REPORT_DATA__',payload))
    print(json.dumps({'crops':len(items),'candidate_crops':len(items) if candidates else 0,
                      'source_sha256':digest,'report':report}))


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--reference');parser.add_argument('--report',action='store_true')
    args=parser.parse_args();prepare(args.reference,args.report)
