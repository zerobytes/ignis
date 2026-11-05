#!/usr/bin/env python3
"""Build an offline research report from appearance_probe/report.json (stdlib only)."""
import csv
import html
import json
from pathlib import Path
import sys

root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('artifacts/appearance-probe')
data = json.loads((root / 'report.json').read_text())
assert data['schema'] == 1
methods = ['full', 'flat', 'distribution32', 'distribution128']
labels = {'full': 'Malha completa', 'flat': 'Plano médio', 'distribution32': 'Distribuição 32', 'distribution128': 'Distribuição 128'}
flat = []
for case in data['cases']:
    for method in case['methods']:
        flat.append(dict(case=case['case'], cells=case['cells_per_side'], triangles=case['cells_per_side']**2*4,
                         method=method['method'], gpu_ms=method['native_timing']['gpu_ms_median'],
                         native_rmse=method['native_error_vs_reference']['rmse_surface'],
                         filtered_rmse=method['filtered_error_vs_reference']['rmse_surface'],
                         interior_rmse=method['filtered_error_vs_reference']['rmse_interior'],
                         boundary_rmse=method['filtered_error_vs_reference']['rmse_boundary'],
                         reference_rmse=case['reference_half_vs_full']['rmse_surface']))
with (root / 'results.csv').open('w', newline='') as file:
    writer = csv.DictWriter(file, fieldnames=list(flat[0]))
    writer.writeheader()
    writer.writerows(flat)

rows = []
for case in data['cases']:
    by = {r['method']: r for r in case['methods']}
    times = [by[m]['native_timing']['gpu_ms_median'] for m in methods]
    fmt = lambda v: 'indisponível' if v is None else f'{v:.4f}'
    rows.append('<tr><td>' + html.escape(case['id']) + '</td>' + ''.join(f'<td>{fmt(t)}<br><span style="color:#8e9bac">{fmt(by[m]["native_timing"]["gpu_ms_p10"])}–{fmt(by[m]["native_timing"]["gpu_ms_p90"])}</span></td>' for m, t in zip(methods, times)) +
                ''.join(f'<td>{by[m]["filtered_error_vs_reference"]["rmse_surface"]:.5f}</td>' for m in methods[1:]) +
                f'<td>{case["reference_half_vs_full"]["rmse_surface"]:.5f}</td></tr>')
mesh_rows = ''.join('<tr>' + ''.join(f'<td>{m[k]:,.3f}</td>' if isinstance(m[k], float) else f'<td>{m[k]:,}</td>' for k in ['triangles','generation_ms','aggregation_ms','upload_and_wait_ms','full_vertex_bytes','proxy_vertex_bytes','distribution32_bytes','distribution128_bytes']) + '</tr>' for m in data['mesh_records'])

payload = json.dumps(data, ensure_ascii=False).replace('<', '\\u003c')
page = r'''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Ignis · Experimento de aparência agregada</title>
<style>
:root{color-scheme:dark;font-family:system-ui,sans-serif;background:#101318;color:#e8edf4}body{max-width:1300px;margin:0 auto;padding:40px 28px}h1{font-size:38px;letter-spacing:-1px;max-width:850px;margin:12px 0}h2{font-size:22px;margin-top:42px}p{line-height:1.6;max-width:1000px;color:#b9c3d0}.tag{color:#ffae6a;font-size:13px;letter-spacing:2px}.panel{border:1px solid #303947;background:#171d25;border-radius:12px;padding:20px;margin:20px 0}.grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:14px}figure{margin:0}figure img{width:100%;image-rendering:auto;border:1px solid #303947;border-radius:6px;background:#20252c}figcaption{font-size:14px;padding:10px 0;color:#c7d1dd}.metrics{font-family:ui-monospace,monospace;font-size:12px;line-height:1.7}select,button,input{font:inherit;background:#232d39;color:inherit;padding:9px;border:1px solid #46566d;border-radius:6px}input[type=range]{width:50%}table{border-collapse:collapse;width:100%;font-size:12px}th,td{text-align:right;padding:9px;border-bottom:1px solid #303947;white-space:nowrap}th:first-child,td:first-child{text-align:left}th{color:#9caec3}a{color:#ffae6a}.scroll{overflow-x:auto}.note{border-left:3px solid #ffae6a;padding-left:18px}.motion{grid-template-columns:repeat(3,minmax(0,1fr))}svg{width:100%;max-width:900px;background:#171d25}code{color:#ffd2a8}@media(max-width:800px){.grid{grid-template-columns:1fr 1fr}h1{font-size:28px}.motion{grid-template-columns:1fr}}
</style>
<div class="tag">IGNIS / PESQUISA GRÁFICA / EAF-62</div>
<h1>Quando microgeometria pode virar aparência?</h1>
<p>Malha procedural completa versus uma superfície de 128 triângulos com resumos de orientação e material. A hipótese é preservar a contribuição visual enquanto o custo de desenho deixa de acompanhar o número original de faces.</p>
<div class="panel"><b id="adapter"></b><p id="overview"></p><p class="note">Laboratório isolado: sem sombras lançadas, iluminação global, hierarquia adaptativa ou correção de auto-oclusão no agregado. O experimento não mede Nanite/Lumen. A referência de 2.048 amostras por pixel tem erro numérico residual.</p></div>
<h2>Comparação visual</h2>
<p>Selecione densidade e condição. A referência acumula 32 grades 8×8 deslocadas por sequência Halton (2.048 amostras por pixel); os candidatos usam 4×4. Todos são reduzidos em espaço linear. Imagens exibidas com Reinhard + sRGB; os erros são medidos antes dessa transformação.</p>
<select id="case"></select><label style="margin-left:12px"><input id="errors" type="checkbox"> Mostrar erros ×8</label>
<div class="panel grid" id="images"></div>
<p id="quality"></p>
<h2>Custo de desenho na mesma resolução</h2>
<p>256×256, uma amostra por pixel em todos os métodos. Mediana de 31 passes GPU após aquecimento. Inclui avaliação das distribuições no vertex shader e clear/store; exclui preparação, upload, readback e PNG. Estes tempos e as imagens filtradas acima têm resoluções de renderização diferentes; não são um único ponto de qualidade/desempenho. O CSV também contém o erro da imagem nativa de cada método.</p>
<div class="panel"><div id="chart"></div></div>
<h2>Câmera e luz em movimento</h2>
<p>12 amostras de uma trajetória curta, 1.048.576 triângulos. A diferença temporal dos resíduos mede a variação do erro em coordenadas de tela, incluindo bordas. Não é uma métrica perceptual de cintilação nem usa compensação de movimento.</p>
<button id="play">Reproduzir</button> <input id="frame" type="range" min="0" max="11" value="0"><span id="frameLabel"></span>
<div class="panel grid motion" id="motion"></div>
<h2>Resultados completos</h2>
<p>Tempos em milissegundos, mediana e faixa p10–p90; RMSE linear sobre a união da superfície visível, sem diluição por fundo vazio. A última coluna mede a diferença entre as referências de 16 e 32 passes; essa diferença é um indicador de convergência, não um limite garantido; valores próximos exigem referência mais refinada.</p>
<div class="panel scroll"><table><thead><tr><th>Caso</th><th>GPU completa</th><th>GPU plano</th><th>GPU dist.32</th><th>GPU dist.128</th><th>RMSE plano</th><th>RMSE dist.32</th><th>RMSE dist.128</th><th>Ref.16/32 passes</th></tr></thead><tbody>__ROWS__</tbody></table></div>
<h2>Preparação e memória explícita</h2>
<p>Uma densidade residente por vez. Bytes abaixo são buffers de geometria/resumos, não memória total do processo. Targets, readback, staging e alocações internas do driver são adicionais. Os dois resumos são construídos juntos; o tempo de agregação inclui ambos.</p>
<div class="panel scroll"><table><thead><tr><th>Triângulos</th><th>Gerar ms</th><th>Agregar ms</th><th>Upload+wait ms</th><th>Malha bytes</th><th>Proxy bytes</th><th>Dist.32 bytes</th><th>Dist.128 bytes</th></tr></thead><tbody>__MESH__</tbody></table></div>
<p><a href="report.json">Dados JSON e protocolo</a> · <a href="results.csv">Resultados CSV</a></p>
<p>Relatório reproduzível: <code>cargo run --locked --release --example appearance_probe</code>, depois <code>python3 scripts/report_appearance_probe.py</code>.</p>
<script>
const data=__DATA__;
const names={full:'Malha completa',flat:'Plano médio',distribution32:'Distribuição 32',distribution128:'Distribuição 128'};
const selector=document.querySelector('#case');
for(const row of data.cases){const o=document.createElement('option');o.value=row.id;o.textContent=row.id;selector.append(o)}
selector.value='n512_far_glossy';
const format=n=>n===null?'indisponível':n.toFixed(5);
function show(){const row=data.cases.find(r=>r.id===selector.value);const error=document.querySelector('#errors').checked;
 document.querySelector('#images').innerHTML=['reference','flat','distribution32','distribution128'].map(m=>{
 const stats=row.methods.find(r=>r.method===m);const file=row.id+'_'+m+(error&&m!=='reference'?'_error':'')+'.png';
 return `<figure><img alt="${m}" src="${file}"><figcaption>${m==='reference'?'Referência integrada':names[m]}</figcaption><div class="metrics">${stats?`RMSE: ${format(stats.filtered_error_vs_reference.rmse_surface)}<br>Interior: ${format(stats.filtered_error_vs_reference.rmse_interior)}<br>Contorno: ${format(stats.filtered_error_vs_reference.rmse_boundary)}<br>Erro nativo: ${format(stats.native_error_vs_reference.rmse_surface)}`:'Geometria completa<br>Integração linear'}</div></figure>`}).join('');
 document.querySelector('#quality').textContent=`Distância ${row.parameters.distance}, inclinação ${row.parameters.tilt}°, luz ${row.parameters.light_angle}°, expoente ${row.parameters.exponent}. Diferença de referência 16/32 passes: ${format(row.reference_half_vs_full.rmse_surface)}.`;
}
selector.onchange=show;document.querySelector('#errors').onchange=show;show();
document.querySelector('#adapter').textContent=data.adapter;
document.querySelector('#overview').textContent=`${data.cases.length} condições estáticas · 12 frames de movimento · ${data.timestamp_period_ns===null?'timestamps GPU indisponíveis':'timestamps GPU disponíveis'} · execução ${data.total_seconds.toFixed(1)} s`;
function motion(){const f=Number(document.querySelector('#frame').value);const frame=data.motion[f];document.querySelector('#frameLabel').textContent=` ${f+1}/12`;
 document.querySelector('#motion').innerHTML=['full','flat','distribution32'].map(m=>{const metric=frame.methods.find(x=>x.method===m);return `<figure><img alt="${m}" src="motion_${String(f).padStart(2,'0')}_${m}.png"><figcaption>${m==='full'?'Referência integrada':names[m]}</figcaption><div class="metrics">${metric?`RMSE: ${format(metric.error.rmse_surface)}<br>Residual temporal: ${format(metric.temporal_residual_rmse)}`:'Geometria completa'}</div></figure>`}).join('');}
document.querySelector('#frame').oninput=motion;motion();let timer=null;document.querySelector('#play').onclick=()=>{if(timer){clearInterval(timer);timer=null;document.querySelector('#play').textContent='Reproduzir';}else{timer=setInterval(()=>{const e=document.querySelector('#frame');e.value=(Number(e.value)+1)%12;motion()},200);document.querySelector('#play').textContent='Pausar';}};
const selected=data.cases.filter(r=>r.case==='far_diffuse');const colors=['#ffb56c','#b4bfcc','#62d8cb','#ad97ff'];
const all=selected.flatMap(r=>r.methods.flatMap(m=>[m.native_timing.gpu_ms_p10,m.native_timing.gpu_ms_p90])).filter(x=>x>0);
if(all.length){const W=880,H=300,L=65,R=25,T=25,B=55;const lo=Math.log10(Math.min(...all)*.7),hi=Math.log10(Math.max(...all)*1.4);const X=n=>L+(Math.log2(n)-5)/4*(W-L-R);const Y=v=>T+(hi-Math.log10(v))/(hi-lo)*(H-T-B);let s=`<svg viewBox="0 0 ${W} ${H}" role="img" aria-label="Tempos GPU por densidade, escala logarítmica">`;
for(let i=0;i<5;i++){let v=10**(lo+(hi-lo)*i/4),y=Y(v);s+=`<line x1="${L}" x2="${W-R}" y1="${y}" y2="${y}" stroke="#303947"/><text x="5" y="${y+4}" fill="#b9c3d0" font-size="11">${v.toFixed(3)} ms</text>`}
['full','flat','distribution32','distribution128'].forEach((m,j)=>{const pts=selected.map(r=>[X(r.cells_per_side),Y(r.methods.find(x=>x.method===m).native_timing.gpu_ms_median)]);s+=`<polyline points="${pts.map(p=>p.join(',')).join(' ')}" fill="none" stroke="${colors[j]}" stroke-width="2"/>`;for(const r of selected){const t=r.methods.find(x=>x.method===m).native_timing;const x=X(r.cells_per_side);s+=`<line x1="${x}" x2="${x}" y1="${Y(t.gpu_ms_p10)}" y2="${Y(t.gpu_ms_p90)}" stroke="${colors[j]}" opacity=".6"/>`;}for(const p of pts)s+=`<circle cx="${p[0]}" cy="${p[1]}" r="4" fill="${colors[j]}"/>`;s+=`<text x="${L+j*195}" y="${H-5}" fill="${colors[j]}" font-size="12">${names[m]}</text>`});for(const r of selected)s+=`<text x="${X(r.cells_per_side)}" y="${H-30}" fill="#b9c3d0" font-size="11" text-anchor="middle">${(r.cells_per_side**2*4).toLocaleString('pt-BR')} faces</text>`;document.querySelector('#chart').innerHTML=s+'</svg>';}
else document.querySelector('#chart').textContent='O adaptador não oferece timestamps; não há ganho GPU medido.';
</script></html>'''
page = page.replace('__ROWS__', ''.join(rows)).replace('__MESH__', mesh_rows).replace('__DATA__', payload)
if (root / 'adaptive' / 'report.json').is_file():
    page = page.replace('<h1>', '<p><a href="adaptive/index.html">Novo: seleção adaptativa com câmera se aproximando e afastando →</a></p><h1>', 1)
if (root / 'multiscale' / 'report.json').is_file():
    page = page.replace('<h1>', '<p><a href="multiscale/index.html">Novo: resumos multiescala com contenção por pixel →</a></p><h1>', 1)
(root / 'index.html').write_text(page)
print(root / 'index.html')

# Optional contact sheet when Pillow is available; HTML/CSV need only the stdlib.
try:
    from PIL import Image, ImageChops, ImageDraw, ImageFont
except ImportError:
    pass
else:
    sheet = Image.new('RGB', (1280, 950), '#101318')
    draw = ImageDraw.Draw(sheet)
    font_path = '/System/Library/Fonts/Supplemental/Arial.ttf'
    font = ImageFont.truetype(font_path, 19) if Path(font_path).exists() else ImageFont.load_default()
    small = ImageFont.truetype(font_path, 15) if Path(font_path).exists() else ImageFont.load_default()
    draw.text((24, 18), 'IGNIS / Aparência agregada — experimento GPU', font=font, fill='#f3f5fa')
    draw.text((24, 49), 'Mesmo recorte ampliado por linha. Cor: Reinhard + sRGB. Erro: diferença linear absoluta ×8.', font=small, fill='#bac5d3')
    choices = [('n32_near_diffuse', '4.096 faces / perto: detalhes ainda visíveis'), ('n512_far_glossy', '1.048.576 faces / longe: brilho subpixel'), ('n512_grazing_view', '1.048.576 faces / vista rasante: visibilidade incompleta')]
    for row, (case_id, title) in enumerate(choices):
        y = 94 + row * 280
        draw.text((24, y), title, font=font, fill='#ffb56c')
        ref = Image.open(root / f'{case_id}_reference.png').convert('RGB')
        bbox = ImageChops.difference(ref, Image.new('RGB', ref.size, ref.getpixel((0, 0)))).getbbox()
        assert bbox
        box = (max(0, bbox[0]-3), max(0, bbox[1]-3), min(ref.width, bbox[2]+3), min(ref.height, bbox[3]+3))
        for col, (suffix, title) in enumerate([('reference', 'Referência integrada'), ('flat', 'Plano médio'), ('distribution32', 'Distribuição 32'), ('distribution32_error', 'Erro da distribuição ×8')]):
            picture = Image.open(root / f'{case_id}_{suffix}.png').convert('RGB').crop(box)
            picture.thumbnail((290, 205), Image.Resampling.LANCZOS)
            # Expand small source crops with the same scale within each row.
            scale = min(290/picture.width, 205/picture.height)
            picture = picture.resize((round(picture.width*scale), round(picture.height*scale)), Image.Resampling.NEAREST)
            x = 24 + col * 315
            sheet.paste(picture, (x+(290-picture.width)//2, y+35+(205-picture.height)//2))
            draw.text((x, y+244), title, font=small, fill='#c6d0de')
    draw.text((24, 925), 'Resultado de uma superfície procedural controlada. Não é uma comparação com Nanite/Lumen.', font=small, fill='#bac5d3')
    sheet.save(root / 'comparison.png')
