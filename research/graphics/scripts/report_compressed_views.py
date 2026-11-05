#!/usr/bin/env python3
"""Compare lossless payload packing and its native filtered images/timings."""
import csv
import json
from pathlib import Path
from statistics import median

ROOT = Path('artifacts/appearance-probe/compression')
data = json.loads((ROOT / 'report.json').read_text())
data['disk_bytes'] = sum((ROOT/f).stat().st_size for f in ['atlas.bin', 'atlas.json'])
summary = {}
rows = data['cases']
for scene in dict.fromkeys(r['scene'] for r in rows):
    cases = [r for r in rows if r['scene'] == scene]
    summary[scene] = {
        'compact_full_gpu_ratio': median(r['timings'][2]['gpu']['median'] / r['timings'][0]['gpu']['median'] for r in cases),
        'original_full_gpu_ratio': median(r['timings'][1]['gpu']['median'] / r['timings'][0]['gpu']['median'] for r in cases),
        'compact_original_gpu_ratio': median(r['timings'][2]['gpu']['median'] / r['timings'][1]['gpu']['median'] for r in cases),
        'compact_full_wall_ratio': median(r['timings'][2]['serial_frame']['median'] / r['timings'][0]['serial_frame']['median'] for r in cases),
        'original_rmse_median': median(r['original_error']['rmse_surface'] for r in cases),
        'compact_rmse_median': median(r['compact_error']['rmse_surface'] for r in cases),
        'delta_rmse_max': max(r['delta']['rmse_surface'] for r in cases),
        'delta_max_rgb': max(r['delta']['max_abs'] for r in cases),
        'delta_alpha_max': max(r['delta']['alpha_rmse'] for r in cases),
        'reference_rmse_increase_max': max(r['compact_error']['rmse_surface'] - r['original_error']['rmse_surface'] for r in cases),
        'temporal_delta_max': max(r['temporal_delta'] or 0 for r in cases),
    }
(ROOT/'summary.json').write_text(json.dumps(summary, indent=2))
with (ROOT/'results.csv').open('w', newline='') as f:
    writer = csv.DictWriter(f, fieldnames=['scene'] + list(next(iter(summary.values()))))
    writer.writeheader()
    writer.writerows(dict(scene=k, **v) for k, v in summary.items())
# Reuse the established visual styling, not its results or runtime logic.
style = (Path('scripts/report_precomputed_views.py').read_text().split('<style>', 1)[1].split('</style>', 1)[0])
page = r'''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Ignis · Compactação de vistas</title><style>__STYLE__</style>
<div class="tag">IGNIS / PESQUISA GRÁFICA / EAF-71</div><h1>As mesmas respostas, em menos memória.</h1>
<p>Faixas ocupadas das vistas são empacotadas no build, com bordas extras para preservar a filtragem. A GPU consulta diretamente o atlas compacto. Não há expansão para75MiB durante o movimento da câmera ou da luz.</p>
<div class="box"><div class="stats" id="memory"></div><p>Payload residente de textura e descritores, incluindo halos e espaços de empacotamento. Uniforms comuns e overhead do driver não estão incluídos. O laboratório mantém as duas representações e a malha para comparação; estes números são o custo de cada representação individual.</p></div>
<div class="box"><div class="controls"><label>Percurso <select id="scene"><option value="both">Câmera + luz</option><option value="camera">Só câmera</option><option value="light">Só luz</option><option value="near">Próximo · câmera + luz</option><option value="seam">Passagem360° →0°</option></select></label><button id="play">Reproduzir</button><button id="back" aria-label="Frame anterior">←</button><button id="next" aria-label="Próximo frame">→</button><label><input type="checkbox" id="zoom" checked> Ampliar região central</label></div><div class="controls"><label for="frame">Posição</label><input id="frame" type="range" min="0" value="0"><span id="position"></span></div><p id="pose"></p></div>
<div class="grid" id="images"></div><p class="note" id="quality"></p>
<div class="box"><h2>Tempo de GPU na posição atual</h2><div id="timings"></div><p>Mesmo256²,1amostra,63 rodadas intercaladas; mediana e p10–p90. “Frame serial” inclui atualização, submissão e espera, sem readback; não equivale a FPS de gameplay. As imagens são capturas nativas reproduzidas nesta interface.</p><p id="aggregate"></p></div>
<div class="box"><h2>O que foi preservado</h2><p>Todos os bits dos texels RGBA16F e das bordas duplicadas foram conferidos contra o atlas original. Nenhum detalhe ou direção de luz foi descartado. A imagem filtrada pode apresentar diferenças numéricas pequenas porque as coordenadas e a localização física das amostras mudaram. O painel de diferença e as métricas expõem essas diferenças; “sem perda nos dados” não significa pixels finais bit a bit idênticos.</p><p>A referência integrada usa256amostras; original e compacto usam16. O erro já existente de reconstrução da ponte, desoclusão e detalhes próximos permanece. O domínio continua sendo24vistas ×24luzes em azimute, elevações fixas55°/35°, material imóvel e luz direcional unitária, sem sombras projetadas/GI/animação.</p><p id="checks"></p><p id="preparation"></p><p><a href="../views/index.html">Experimento original</a> · <a href="../quality/index.html">Seleção por qualidade</a> · <a href="summary.json">Resumo medido</a> · <a href="results.csv">CSV</a></p></div>
<script>
const data=__DATA__,summary=__SUMMARY__,$=id=>document.getElementById(id);let timer=null,token=0;
const fmt=(x,n=5)=>x==null?'indisponível':Number(x).toFixed(n).replace('.',',');
const series=()=>data.cases.filter(r=>r.scene===$('scene').value);
function stop(){if(timer)clearInterval(timer);timer=null;$('play').textContent='Reproduzir'}
async function show(){const seq=++token,rows=series();$('frame').max=rows.length-1;const i=Math.min(Number($('frame').value),rows.length-1);$('frame').value=i;const r=rows[i];$('position').textContent=`${i+1}/${rows.length}`;
$('pose').textContent=`Câmera ${fmt(r.pose.camera,2)}° · luz ${fmt(r.pose.light,2)}° · distância ${fmt(r.pose.distance,1)}`;
const panels=[['reference','Referência integrada · geometria'],['original','Tabela original ·75MiB'],['compact',`Tabela compacta ·${fmt(data.packed_bytes/1048576,2)}MiB`],['delta','Diferença compacto/original ×8']];
const figures=await Promise.all(panels.map(async([suffix,label])=>{const img=new Image();await new Promise((resolve,reject)=>{img.onload=resolve;img.onerror=reject;img.src=`${r.id}_${suffix}.png`});const figure=document.createElement('figure'),canvas=document.createElement('canvas'),caption=document.createElement('figcaption');canvas.width=canvas.height=256;const w=$('zoom').checked?(r.pose.distance<5?210:85):256,x=(256-w)/2;canvas.getContext('2d').drawImage(img,x,x,w,w,0,0,256,256);caption.textContent=label;figure.append(canvas,caption);return figure}));if(seq!==token)return;$('images').replaceChildren(...figures);
$('quality').textContent=`Erro introduzido pela consulta compacta: RMSE RGB ${fmt(r.delta.rmse_surface,8)} · máximo ${fmt(r.delta.max_abs,6)} · cobertura ${fmt(r.delta.alpha_rmse,8)}. Contra geometria: original ${fmt(r.original_error.rmse_surface,6)}, compacto ${fmt(r.compact_error.rmse_surface,6)}. Diferença temporal ${fmt(r.temporal_delta,8)}. Em1amostra: RMSE ${fmt(r.native_delta.rmse_surface,8)}, máximo ${fmt(r.native_delta.max_abs,6)}.`;
$('timings').innerHTML='<table><tr><th>Método</th><th>GPU ms</th><th>p10–p90</th><th>Frame serial ms</th></tr>'+['Geometria completa','Tabela original','Tabela compacta'].map((name,j)=>{const t=r.timings[j];return `<tr><td>${name}</td><td>${fmt(t.gpu.median)}</td><td>${fmt(t.gpu.p10)}–${fmt(t.gpu.p90)}</td><td>${fmt(t.serial_frame.median)}</td></tr>`}).join('')+'</table>';
const a=summary[r.scene];$('aggregate').textContent=`Percurso: compacto/geometria ${fmt(a.compact_full_gpu_ratio,3)}× na GPU; compacto/original ${fmt(a.compact_original_gpu_ratio,3)}×. Frame serial compacto/geometria ${fmt(a.compact_full_wall_ratio,3)}×. São medianas das razões por pose. Maior RMSE introduzido: ${fmt(a.delta_rmse_max,8)}.`;
}
function step(d){$('frame').value=(Number($('frame').value)+d+series().length)%series().length;show()}
$('frame').oninput=()=>{stop();show()};for(const id of ['scene','zoom'])$(id).onchange=()=>{stop();show()};$('back').onclick=()=>{stop();step(-1)};$('next').onclick=()=>{stop();step(1)};$('play').onclick=()=>{if(timer)stop();else{$('play').textContent='Pausar';timer=setInterval(()=>step(1),180)}};
$('memory').innerHTML=[[fmt(data.source_bytes/1048576,2)+'MiB','original'],[fmt(data.packed_bytes/1048576,2)+'MiB','compacto'],[fmt((1-data.packed_bytes/data.source_bytes)*100,1)+'%','memória economizada']].map(([v,l])=>`<span><b>${v}</b>${l}</span>`).join('');
$('preparation').textContent=`Empacotamento e verificação CPU ${fmt(data.build_seconds,3)}s. Carregamento original ${fmt(data.original_load_ms,2)}ms; compacto ${fmt(data.compact_load_ms,2)}ms incluindo pipeline específico, validação e upload. Disco ${fmt(data.disk_bytes/1048576,2)}MiB com metadados JSON. O tempo do bake original é separado. ${data.adapter}.`;
$('checks').textContent=`${data.cases.length}poses; ${data.checks.bounded_pairs}comparações de retângulo/tela inteira exatas. Repetição, retorno de luz e equivalência+360° exatos. A inspeção visual e o aceite humano são separados destes testes.`;show();
</script></html>'''
(ROOT/'index.html').write_text(page.replace('__STYLE__', style).replace('__DATA__', json.dumps(data).replace('<', '\\u003c')).replace('__SUMMARY__', json.dumps(summary)))
print(ROOT/'index.html')
try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    pass
else:
    selected = [max((r for r in rows if r['scene']==s), key=lambda r:r['delta']['rmse_surface']) for s in ['camera','both','near']]
    image=Image.new('RGB',(1280,1090),'#0c121b');draw=ImageDraw.Draw(image)
    font=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',18)
    draw.text((24,20),'IGNIS / compactacao sem perdas nos texels / piores RMSE por percurso',font=font,fill='white')
    for row,r in enumerate(selected):
        y=75+row*330;draw.text((24,y),f"{r['id']} / delta RMSE {r['delta']['rmse_surface']:.8f}",font=font,fill='#ffb66b')
        for col,(suffix,label) in enumerate([('reference','Referencia'),('original','Original'),('compact','Compacto'),('delta','Diferenca x8')]):
            src=Image.open(ROOT/f"{r['id']}_{suffix}.png");w=210 if r['pose']['distance']<5 else 85;x=(256-w)//2
            src=src.crop((x,x,x+w,x+w)).resize((250,250),Image.Resampling.NEAREST);image.paste(src,(24+col*312,y+32));draw.text((24+col*312,y+292),label,font=font,fill='white')
    image.save(ROOT/'comparison.png')
