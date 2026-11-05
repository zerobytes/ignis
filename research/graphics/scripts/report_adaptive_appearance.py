#!/usr/bin/env python3
"""Offline, inspectable camera-path report. Uses only the Python standard library."""
import csv
import json
from pathlib import Path
import sys

root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('artifacts/appearance-probe/adaptive')
data = json.loads((root / 'report.json').read_text())
assert data['schema'] == 1
rows = []
for r in data['cases']:
    rows.append(dict(case=r['case'], frame=r['frame'], direction=r['direction'], distance=r['parameters']['distance'],
                     detailed_regions=r['detail_regions'], triangles=r['triangles'], draw_calls=r['draw_calls'],
                     changed_regions=r['changed_regions'], selection_cpu_us=r['selection_cpu_us'],
                     adaptive_rmse=r['adaptive_error']['rmse_surface'], full_filtered_rmse=r['full_filtered_error']['rmse_surface'],
                     summary_rmse=r['summary_error']['rmse_surface'], convergence_rmse=r['reference_half_vs_full']['rmse_surface'],
                     switch_same_pose_rmse=r['switch_same_pose_rmse'], temporal_residual_rmse=r['temporal_residual_rmse'],
                     native_gpu_ms=r['native_timing']['gpu_ms_median'], native_full_gpu_ms=r['native_full_timing']['gpu_ms_median']))
with (root / 'results.csv').open('w', newline='') as f:
    writer = csv.DictWriter(f, fieldnames=list(rows[0]))
    writer.writeheader()
    writer.writerows(rows)
page = r'''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Ignis · Seleção adaptativa</title><style>
:root{color-scheme:dark;font-family:system-ui,sans-serif;background:#0d1219;color:#e8eef7}body{max-width:1400px;margin:0 auto;padding:30px 24px}h1{font-size:38px;letter-spacing:-1px;margin:12px 0}h2{font-size:23px;margin-top:36px}p{line-height:1.65;color:#b6c5d5;max-width:1100px}.tag{color:#ffaa55;letter-spacing:2px;font-size:12px}.panel{background:#151e2a;border:1px solid #2c3a4c;border-radius:12px;padding:18px;margin:20px 0}.controls{display:flex;flex-wrap:wrap;gap:12px;align-items:center}select,button,input{font:inherit;color:inherit;background:#202e40;border:1px solid #425775;border-radius:6px;padding:8px}input[type=range]{flex:1;min-width:180px;padding:0}label{font-size:14px}.grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:16px}figure{margin:0}figure img{width:100%;background:#1e2530;border:1px solid #34455c;border-radius:6px}figcaption{font-size:14px;line-height:1.5;margin-top:9px}.metrics{font-family:ui-monospace,monospace;font-size:12px;color:#b4c7df;margin-top:8px;line-height:1.7}.pixel img{image-rendering:pixelated}.stats{display:flex;flex-wrap:wrap;gap:25px}.stats b{font-size:27px;display:block;color:#fff}.stats span{font-size:13px;color:#a9bcd2}.blue{color:#62beff}.orange{color:#ffad57}svg{width:100%;max-width:1000px}a{color:#ffad57}.legend{display:flex;gap:24px;flex-wrap:wrap}.region{display:grid;grid-template-columns:repeat(8,1fr);gap:3px;max-width:360px}.region span{font-size:11px;padding:6px 2px;text-align:center;color:#07101a;border-radius:3px}.split{display:grid;grid-template-columns:2fr 1fr;gap:22px}.note{border-left:3px solid #ffad57;padding-left:15px}code{color:#ffcf9e}@media(max-width:850px){.grid{grid-template-columns:1fr 1fr}.split{grid-template-columns:1fr}h1{font-size:28px}}@media(max-width:500px){.grid{grid-template-columns:1fr}}
</style><div class="tag">IGNIS / LABORATÓRIO GRÁFICO</div><h1>Geometria perto. Resumo longe.</h1>
<p>A câmera percorre as mesmas posições nos dois sentidos. Cada uma das 64 regiões mantém sua microgeometria enquanto o detalhe ainda ocupa espaço na imagem e passa à distribuição de aparência quando ele se torna subpixel.</p>
<div class="panel"><div class="controls"><label>Cena <select id="scene"><option value="diffuse">Difusa · 35°</option><option value="glossy" selected>Brilhante · 35°</option><option value="grazing">Rasante · 75°</option></select></label><button id="play">Reproduzir ida e volta</button><button id="back" aria-label="Frame anterior">←</button><button id="next" aria-label="Próximo frame">→</button><label><input id="zoom" type="checkbox"> Pixels nítidos</label><label><input id="errors" type="checkbox"> Erro ×8</label></div>
<div class="controls" style="margin-top:20px"><label for="frame">Câmera</label><input id="frame" type="range" min="0" max="48" value="0"><span id="position"></span></div><p id="pose" aria-live="polite"></p><div class="stats" id="stats"></div></div>
<div class="grid" id="images"></div>
<p class="legend"><span class="blue">Azul: geometria completa</span><span class="orange">Laranja: resumo de aparência</span></p>
<p>Imagens calculadas na GPU: referência com 2.048 amostras por pixel; candidatos com 16. As imagens de 256×256 são ampliadas para inspeção. O seletor usa os pixels da imagem final, sem contar as amostras extras de filtragem.</p>
<div class="panel split"><div><h2 style="margin-top:0">O caminho da decisão</h2><div id="chart"></div><p id="opposite"></p></div><div><h2 style="margin-top:0">Detalhe projetado / região</h2><div class="region" id="regions"></div><p>Limite conservador do diâmetro da microcélula em pixels. A grade representa regiões do objeto, não a projeção da tela.</p></div></div>
<div class="panel"><h2 style="margin-top:0">O que acontece na troca?</h2><p id="quality"></p><p id="timing"></p><p id="jitter"></p></div>
<h2>Regra deste experimento</h2><p>Entrar no resumo: abaixo de <b>0,75 pixel</b>. Retornar à geometria: acima de <b>1 pixel</b>. Entre esses valores, conservar a escolha anterior. Essa histerese dá estabilidade; a troca continua discreta e pode produzir um salto de aparência. Uma região que intercepta o plano próximo da câmera mantém geometria.</p>
<p class="note">O limite matemático controla tamanho geométrico projetado. Ele não limita erro de cor, brilho, contorno ou visibilidade. O resumo usa 32 grupos de orientação/material e não corrige auto-oclusão. Veja especialmente o caso rasante e o erro ×8.</p>
<p>Os tempos GPU são medianas de 31 passes nativos a uma amostra por pixel, após aquecimento; não correspondem ao custo das imagens filtradas acima. Excluem seleção CPU, upload, readback e relatório. A medição de CPU é uma única atualização de 64 regiões, sujeita a ruído. Malha e resumo continuam residentes; reduzir triângulos desenhados não demonstra economia de memória ou streaming.</p>
<p>O salto compara a seleção anterior e a nova na <b>mesma pose</b>, com o mesmo filtro. O resíduo temporal compara frames sucessivos em coordenadas de tela, sem compensação de movimento. A convergência de 16/32 passes da referência é um diagnóstico, não um limite certificado de erro.</p>
<p id="metadata"></p><p><a href="report.json">Dados JSON</a> · <a href="results.csv">CSV completo</a> · <a href="../index.html">Experimento de aparência inicial</a></p>
<p>Reproduzir: <code>cargo run --locked --release --example appearance_probe -- --adaptive</code> e <code>python3 scripts/report_adaptive_appearance.py</code>.</p>
<script>
const data=__DATA__, $=id=>document.getElementById(id), fmt=(x,n=5)=>x==null?'indisponível':x.toFixed(n);
const series=()=>data.cases.filter(r=>r.case===$('scene').value);let timer=null,request=0;
function stop(){clearInterval(timer);timer=null;$('play').textContent='Reproduzir ida e volta'}
async function show(){const ticket=++request,rows=series(),i=Number($('frame').value),r=rows[i],op=rows[48-i];
const panels=[['reference','Referência integrada',`2.048 amostras/pixel · residual ${fmt(r.reference_half_vs_full.rmse_surface)}`],[$('errors').checked?'error':'adaptive','Seleção adaptativa',`RMSE ${fmt(r.adaptive_error.rmse_surface)} · contorno ${fmt(r.adaptive_error.rmse_boundary)}`],['summary','Sempre resumo',`RMSE ${fmt(r.summary_error.rmse_surface)}`],['selection','Escolha por região',`${r.detail_regions} geometria · ${r.summary_regions} resumo`]];
const figures=await Promise.all(panels.map(async([suffix,title,metric])=>{const figure=document.createElement('figure'),img=new Image();img.alt=title;await new Promise(resolve=>{img.onload=resolve;img.onerror=()=>{img.alt=title+' — imagem indisponível';resolve()};img.src=r.id+'_'+suffix+'.png'});figure.append(img);const caption=document.createElement('figcaption');caption.textContent=title;figure.append(caption);const m=document.createElement('div');m.className='metrics';m.textContent=metric;figure.append(m);return figure}));
if(ticket!==request)return;$('images').replaceChildren(...figures);$('images').classList.toggle('pixel',$('zoom').checked);
$('position').textContent=`${i+1} / 49`;$('pose').textContent=`${r.direction.toUpperCase()} · distância ${r.parameters.distance.toFixed(2)} · inclinação ${r.parameters.tilt}° · mesma cena e luz em todo o percurso`;
$('stats').innerHTML=[[r.triangles.toLocaleString('pt-BR'),'triângulos desenhados'],[r.detail_regions+'/64','regiões com geometria'],[r.draw_calls,'chamadas de desenho'],[r.changed_regions,'regiões trocadas neste passo']].map(([v,t])=>`<span><b>${v}</b>${t}</span>`).join('');
$('regions').innerHTML=r.footprint_px.map((p,j)=>`<span style="background:${r.detailed[j]?'#62beff':'#ffad57'}" title="Região ${j}: ${r.detailed[j]?'geometria':'resumo'}">${p.toFixed(2)}</span>`).join('');
$('opposite').textContent=`Na mesma posição no percurso inverso: ${op.detail_regions} regiões com geometria e ${op.triangles.toLocaleString('pt-BR')} triângulos. A diferença dentro da faixa de 0,75–1 pixel é intencional: a decisão conserva seu histórico.`;
$('quality').textContent=`Salto provocado só pela troca: RMSE ${fmt(r.switch_same_pose_rmse)}. Resíduo temporal: ${fmt(r.temporal_residual_rmse)}. Geometria completa filtrada: RMSE ${fmt(r.full_filtered_error.rmse_surface)}. Todos medidos em RGB linear sobre a superfície, antes de Reinhard/sRGB.`;
const t=r.native_timing;$('timing').textContent=`GPU nativa adaptativa: ${fmt(t.gpu_ms_median,4)} ms (p10 ${fmt(t.gpu_ms_p10,4)}–p90 ${fmt(t.gpu_ms_p90,4)}). Malha completa: ${fmt(r.native_full_timing.gpu_ms_median,4)} ms. Seleção CPU: ${fmt(r.selection_cpu_us,2)} µs. Tempos desta execução, sem promessa de FPS.`;
const W=900,H=280,L=65,R=25,T=20,B=45,X=d=>L+(d-2.5)/13.5*(W-L-R),Y=t=>T+(1-t/65536)*(H-T-B);let svg=`<svg viewBox="0 0 ${W} ${H}" role="img" aria-label="Triângulos desenhados pela distância e sentido da câmera">`;
for(const y of [0,32768,65536])svg+=`<line x1="${L}" x2="${W-R}" y1="${Y(y)}" y2="${Y(y)}" stroke="#35465a"/><text x="0" y="${Y(y)+5}" fill="#b6c5d5" font-size="12">${y}</text>`;
for(const [a,b,c] of [[0,25,'#ffad57'],[24,49,'#62beff']])svg+=`<polyline points="${rows.slice(a,b).map(q=>X(q.parameters.distance)+','+Y(q.triangles)).join(' ')}" fill="none" stroke="${c}" stroke-width="3"/>`;
svg+=`<circle cx="${X(r.parameters.distance)}" cy="${Y(r.triangles)}" r="7" fill="white"/>`;
for(const d of [2.5,6,10,16])svg+=`<text x="${X(d)}" y="${H-15}" text-anchor="middle" fill="#b6c5d5" font-size="13">${d}</text>`;
$('chart').innerHTML=svg+'</svg><div class="legend"><span class="orange">Afastando →</span><span class="blue">← Aproximando</span></div>';
}
function step(delta){$('frame').value=(Number($('frame').value)+delta+49)%49;show()}
$('scene').onchange=()=>{stop();show()};$('frame').oninput=()=>{stop();show()};$('errors').onchange=show;$('zoom').onchange=show;$('back').onclick=()=>{stop();step(-1)};$('next').onclick=()=>{stop();step(1)};
$('play').onclick=()=>{if(timer)stop();else{$('play').textContent='Pausar';timer=setInterval(()=>step(1),400)}};
$('jitter').textContent=`Controle de oscilação: 40 posições alternadas próximas ao limiar. Após inicialização, ${data.jitter.hysteresis_changes_after_initialization} trocas de região com histerese versus ${data.jitter.hard_threshold_changes_after_initialization} com limiar único de 0,875 pixel. É um teste do seletor, não uma medição visual de cintilação.`;
$('metadata').textContent=`${data.adapter} · ${data.cases.length} capturas de trajetória · execução ${data.total_seconds.toFixed(1)} s · 65.536 triângulos de origem / 128 no resumo integral.`;show();
</script></html>'''
page = page.replace('__DATA__', json.dumps(data, ensure_ascii=False).replace('<', '\\u003c'))
if (root.parent / 'multiscale' / 'report.json').is_file():
    page = page.replace('<h1>', '<p><a href="../multiscale/index.html">Novo: resumos multiescala com contenção por pixel →</a></p><h1>', 1)
if (root.parent / 'quality' / 'report.json').is_file():
    page = page.replace('<h1>', '<p class="note"><a href="../quality/index.html">Abrir comparação nova: qualidade configurável e transição gradual →</a></p><h1>', 1)
(root / 'index.html').write_text(page)
print(root / 'index.html')

# Optional export for inspection/sharing; the interactive report needs no Pillow.
try:
    from PIL import Image, ImageChops, ImageDraw, ImageFont
except ImportError:
    pass
else:
    canvas = Image.new('RGB', (1280, 1090), '#101722')
    draw = ImageDraw.Draw(canvas)
    font_path = '/System/Library/Fonts/Supplemental/Arial.ttf'
    try:
        font = ImageFont.truetype(font_path, 18)
        small = ImageFont.truetype(font_path, 15)
        title = ImageFont.truetype(font_path, 28)
    except OSError:
        font = small = title = ImageFont.load_default()
    draw.text((24, 18), 'IGNIS / seleção adaptativa por tamanho projetado', font=title, fill='#e8eef7')
    draw.text((24, 58), 'Recortes ampliados igualmente por linha. Azul: geometria; laranja: resumo.', font=small, fill='#b6c5d5')
    labels = ['Referência integrada', 'Adaptativo', 'Sempre resumo', 'Seleção por região']
    cases = [('glossy', 0), ('glossy', 14), ('glossy', 24)]
    for row, (name, frame) in enumerate(cases):
        r = next(r for r in data['cases'] if r['case'] == name and r['frame'] == frame)
        ref = Image.open(root / (r['id'] + '_reference.png')).convert('RGB')
        bbox = ImageChops.difference(ref, Image.new('RGB', ref.size, ref.getpixel((0, 0)))).getbbox()
        x0, y0, x1, y1 = bbox
        pad = 4
        crop = (max(0, x0-pad), max(0, y0-pad), min(256, x1+pad), min(256, y1+pad))
        y = 105 + row * 325
        draw.text((24, y), f'Distância {r["parameters"]["distance"]:.2f} / {r["triangles"]:,} triângulos / {r["detail_regions"]} regiões detalhadas', font=font, fill='#ffad57')
        for col, (suffix, label) in enumerate(zip(['reference','adaptive','summary','selection'], labels)):
            img = Image.open(root / (r['id'] + '_' + suffix + '.png')).crop(crop)
            img.thumbnail((292, 250), Image.Resampling.NEAREST)
            # thumbnail only shrinks; explicitly enlarge small subpixel scenes for inspection.
            ratio = min(292/img.width, 250/img.height)
            img = img.resize((round(img.width*ratio), round(img.height*ratio)), Image.Resampling.NEAREST)
            x = 24 + col*312
            canvas.paste(img, (x+(292-img.width)//2, y+35+(250-img.height)//2))
            draw.text((x, y+291), label, font=small, fill='#b6c5d5')
    canvas.save(root / 'comparison.png')
