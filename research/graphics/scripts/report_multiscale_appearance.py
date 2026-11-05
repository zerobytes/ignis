#!/usr/bin/env python3
"""Reproducible offline comparison of spatial summaries; stdlib, optional Pillow export."""
import csv
import json
from pathlib import Path
import sys

root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('artifacts/appearance-probe/multiscale')
data = json.loads((root/'report.json').read_text())
assert data['schema'] == 1
rows = []
for r in data['cases']:
    row = {k:r[k] for k in ['case','frame','direction','triangles','geometry_cells','summary_nodes','draw_calls','visited_nodes','max_summary_span_px','selection_cpu_us','index_validation_and_queue_us','index_upload_bytes','previous_triangles','switch_same_pose_rmse','temporal_residual_rmse']}
    row['distance'] = r['parameters']['distance']
    row['reference_residual'] = r['reference_half_vs_full']['rmse_surface']
    for m in r['methods']:
        for label, value in [('rmse',m['filtered_error']['rmse_surface']),('alpha_rmse',m['filtered_error']['alpha_rmse']),('native_rmse',m['native_error']['rmse_surface']),('gpu_ms',m['native_timing']['gpu_ms_median'])]:
            row[m['method']+'_'+label] = value
    rows.append(row)
with (root/'results.csv').open('w',newline='') as f:
    writer=csv.DictWriter(f,fieldnames=list(rows[0]));writer.writeheader();writer.writerows(rows)
page=r'''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Ignis · Resumos multiescala</title><style>
:root{color-scheme:dark;font-family:system-ui,sans-serif;background:#0d1219;color:#e8eef7}body{max-width:1400px;margin:0 auto;padding:30px 24px}h1{font-size:38px;letter-spacing:-1px;margin:12px 0}h2{font-size:23px;margin-top:32px}p{line-height:1.65;color:#b6c5d5;max-width:1100px}.tag{color:#ffaa55;letter-spacing:2px;font-size:12px}.panel{background:#151e2a;border:1px solid #2c3a4c;border-radius:12px;padding:18px;margin:20px 0}.controls{display:flex;flex-wrap:wrap;gap:12px;align-items:center}select,button,input{font:inherit;color:inherit;background:#202e40;border:1px solid #425775;border-radius:6px;padding:8px}input[type=range]{flex:1;min-width:180px;padding:0}label{font-size:14px}.grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:16px}figure{margin:0}canvas{width:100%;max-width:256px;border:1px solid #34455c;background:#20252d;border-radius:6px}figcaption{font-size:14px;line-height:1.5;margin-top:9px}.metrics{font-family:ui-monospace,monospace;font-size:12px;color:#b4c7df;margin-top:8px;line-height:1.7}.stats{display:flex;flex-wrap:wrap;gap:25px}.stats b{font-size:27px;display:block;color:#fff}.stats span{font-size:13px;color:#a9bcd2}svg{width:100%;max-width:1100px}a{color:#ffad57}.note{border-left:3px solid #ffad57;padding-left:15px}.split{display:grid;grid-template-columns:1fr 2fr;gap:22px}table{border-collapse:collapse;font-size:13px;width:100%}th,td{padding:9px;text-align:right;border-bottom:1px solid #34455c}th:first-child,td:first-child{text-align:left}code{color:#ffcf9e}summary{cursor:pointer}footer{font-size:13px;color:#a9bcd2;margin-top:30px}@media(max-width:950px){.grid{grid-template-columns:1fr 1fr}.split{grid-template-columns:1fr}h1{font-size:28px}}@media(max-width:500px){.grid{grid-template-columns:1fr}}
</style><div class="tag">IGNIS / PESQUISA GRÁFICA / EAF-65</div><h1>Resumir dentro de um pixel.</h1>
<p>Uma hierarquia conserva regiões pequenas e reúne quatro regiões menores quando o volume inteiro cabe no mesmo pixel da imagem final. A comparação mantém a mesma superfície, câmera e iluminação nos quatro painéis.</p>
<div class="panel"><div class="controls"><label>Cena <select id="scene"><option value="diffuse">Difusa · 35°</option><option value="glossy" selected>Brilhante · 35°</option><option value="grazing">Rasante · 75°</option></select></label><button id="play">Reproduzir ida e volta</button><button id="back" aria-label="Frame anterior">←</button><button id="next" aria-label="Próximo frame">→</button><label><input id="zoom" type="checkbox"> Ampliar objeto</label><label><input id="pixels" type="checkbox"> Grade de pixels ampliados</label><label><input id="errors" type="checkbox"> Erro multiescala ×8</label></div>
<div class="controls" style="margin-top:18px"><label for="frame">Câmera</label><input id="frame" type="range" min="0" value="0"><span id="position"></span></div><p id="pose" aria-live="polite"></p><div class="stats" id="stats"></div></div>
<div class="grid" id="images"></div><p id="sampling"></p>
<p class="note">Cada quadrado colorido é uma região do objeto com muitos triângulos. Os pixels são as células da imagem de 256×256. Use <b>Ampliar objeto + Grade de pixels ampliados</b> para distinguir essas duas escalas. Todos os painéis usam o mesmo recorte; a ampliação não recalcula a seleção nem cria detalhes novos.</p>
<div class="controls"><label>Último painel <select id="control"><option value="fixed">Sempre resumo fixo</option><option value="full">Geometria completa filtrada</option></select></label></div>
<div class="panel"><h2 style="margin-top:0">Trabalho desenhado durante o percurso</h2><div id="chart"></div><p id="opposite"></p><p id="quality"></p></div>
<details class="panel"><summary><b>Diagnóstico da seleção — cores artificiais, separado da aparência</b></summary><div class="split" style="margin-top:20px"><div id="diagnostic"></div><div><p>Azul: geometria original. Laranja: algum nível de resumo. Este mapa localiza as decisões; sua cor não representa material ou iluminação.</p><div id="levels"></div><p id="gate"></p></div></div></details>
<div class="panel"><h2 style="margin-top:0">Qualidade e custo nesta posição</h2><div id="measurements"></div><p id="cpu"></p></div>
<h2>O que esta regra garante</h2><p>A projeção dos oito cantos do volume de uma região, incluindo sua altura, deve permanecer dentro de um único pixel. Entramos no resumo com extensão menor que0,75 pixel e conservamos até1 pixel, desde que a região continue no mesmo pixel. Ao cruzar uma fronteira, a seleção refina imediatamente. Isso preserva separação espacial, mas não garante a mesma iluminação, cobertura ou oclusão.</p>
<p>O resumo guarda orientação e material por área. O proxy continua plano: pode alterar a área visível e perder auto-oclusão. A regra rígida de fronteira também pode aumentar trocas com o movimento. Os gráficos e erros mostram o resultado medido, inclusive casos em que o método anterior permanece melhor.</p>
<p id="memory"></p><p>Tempos GPU medem31 passes a uma amostra por pixel, após aquecimento, com ordem alternada entre métodos. Eles excluem seleção CPU, validação/upload de índices, readback e PNG; não são o custo das imagens filtradas. Ambos os caminhos mantêm a geometria e os resumos na memória. O protótipo usa até dois draws, mas seleciona e envia índices na CPU.</p>
<p>A referência usa2.048 amostras por pixel; seu residual16/32 passes é um diagnóstico de convergência, não erro máximo certificado. O controle completo filtrado ajuda a enxergar erro de amostragem ainda presente nos candidatos. RMSE usa RGB linear sobre a superfície, antes de Reinhard+sRGB.</p>
<p><a href="report.json">Dados JSON</a> · <a href="results.csv">CSV completo</a> · <a href="../adaptive/index.html">Experimento anterior</a></p>
<footer id="metadata"></footer><p>Reproduzir: <code>cargo run --locked --release --example appearance_probe -- --multiscale</code> e <code>python3 scripts/report_multiscale_appearance.py</code>.</p>
<script>
const data=__DATA__,$=id=>document.getElementById(id),fmt=(x,n=5)=>x==null?'indisponível':x.toFixed(n),names={full:'Geometria completa filtrada',fixed:'Sempre resumo fixo',previous:'Seleção anterior · regiões fixas',multiscale:'Resumos multiescala'};
const series=()=>data.cases.filter(r=>r.case===$('scene').value);let timer=null,request=0;const cache=new Map();
$('frame').max=data.frames_per_case-1;
function stop(){clearInterval(timer);timer=null;$('play').textContent='Reproduzir ida e volta'}
function load(src){if(cache.has(src)){const value=cache.get(src);cache.delete(src);cache.set(src,value);return value;}const value=new Promise((resolve,reject)=>{const img=new Image();img.onload=()=>resolve(img);img.onerror=()=>{cache.delete(src);reject(new Error('Imagem indisponível: '+src))};img.src=src});cache.set(src,value);while(cache.size>64)cache.delete(cache.keys().next().value);return value;}
function canvas(img,r,title){const c=document.createElement('canvas');c.width=c.height=256;c.setAttribute('role','img');c.setAttribute('aria-label',title);const ctx=c.getContext('2d');ctx.imageSmoothingEnabled=false;
let x=0,y=0,w=256,h=256;if($('zoom').checked){const b=r.screen_bounds;const side=Math.min(256,Math.ceil(Math.max(b[2]-b[0],b[3]-b[1]))+6);x=Math.max(0,Math.min(256-side,Math.floor((b[0]+b[2]-side)/2)));y=Math.max(0,Math.min(256-side,Math.floor((b[1]+b[3]-side)/2)));w=h=side;}ctx.drawImage(img,x,y,w,h,0,0,256,256);
if($('pixels').checked&&$('zoom').checked&&w<=90){ctx.strokeStyle='rgba(255,255,255,.2)';ctx.lineWidth=.5;ctx.beginPath();for(let i=0;i<=w;i++){const p=i*256/w;ctx.moveTo(p,0);ctx.lineTo(p,256);ctx.moveTo(0,p);ctx.lineTo(256,p)}ctx.stroke();}return c;}
async function show(){const ticket=++request,rows=series(),i=Number($('frame').value),r=rows[i],op=rows[rows.length-1-i];const stats=m=>r.methods.find(x=>x.method===m);const methods=['reference','multiscale','previous',$('control').value];
try{const figures=await Promise.all(methods.map(async m=>{const suffix=m==='multiscale'&&$('errors').checked?'error':m,img=await load(r.id+'_'+suffix+'.png');const fig=document.createElement('figure'),title=suffix==='error'?'Erro multiescala ×8':m==='reference'?'Referência integrada':names[m];fig.append(canvas(img,r,title));const cap=document.createElement('figcaption');cap.textContent=title;fig.append(cap);const text=document.createElement('div');text.className='metrics';text.textContent=m==='reference'?`Residual da referência: ${fmt(r.reference_half_vs_full.rmse_surface)}`:`RMSE ${fmt(stats(m).filtered_error.rmse_surface)} · contorno ${fmt(stats(m).filtered_error.rmse_boundary)}`;fig.append(text);return fig;}));const diagnostic=canvas(await load(r.id+'_diagnostic.png'),r,'Mapa de seleção: azul geometria, laranja resumo');if(ticket!==request)return;$('images').replaceChildren(...figures);$('diagnostic').replaceChildren(diagnostic);
}catch(e){if(ticket===request)$('pose').textContent=e.message;return;}
$('position').textContent=`${i+1} / ${rows.length}`;$('pose').textContent=`${r.direction.toUpperCase()} · distância ${r.parameters.distance.toFixed(3)} · ${r.parameters.tilt}° · ${$('zoom').checked?'objeto ampliado em pixels existentes':'imagem na resolução nativa'}`;
$('stats').innerHTML=[[r.triangles.toLocaleString('pt-BR'),'triângulos multiescala'],[r.previous_triangles.toLocaleString('pt-BR'),'triângulos método anterior'],[r.summary_nodes.toLocaleString('pt-BR'),'resumos selecionados'],[r.draw_calls,'draws multiescala']].map(([v,t])=>`<span><b>${v}</b>${t}</span>`).join('');
$('sampling').textContent=`Referência: ${data.reference_samples_per_pixel.toLocaleString('pt-BR')} amostras/pixel. Todos os candidatos: ${data.candidate_samples_per_pixel} amostras/pixel com a mesma sequência deslocada. A seleção permanece baseada em256×256 pixels finais.`;
$('levels').innerHTML='<table><tr><th>Região resumida</th><th>Nós escolhidos</th></tr>'+r.summaries_by_cell_width.map((n,j)=>`<tr><td>${2**j}×${2**j} microcélulas</td><td>${n.toLocaleString('pt-BR')}</td></tr>`).join('')+`<tr><td>Pirâmides com geometria</td><td>${r.geometry_cells.toLocaleString('pt-BR')}</td></tr></table>`;
$('gate').textContent=`Maior extensão de um resumo escolhido: ${fmt(r.max_summary_span_px,3)} pixel. Todos passaram na contenção em um único pixel. Nós visitados: ${r.visited_nodes.toLocaleString('pt-BR')}.`;
$('quality').textContent=`Troca na mesma pose: RMSE ${fmt(r.switch_same_pose_rmse)}. Resíduo temporal em coordenadas de tela: ${fmt(r.temporal_residual_rmse)}. A primeira métrica muda só a seleção; a segunda inclui o movimento e não é compensada por fluxo óptico.`;
$('opposite').textContent=`Mesma pose no sentido inverso: ${op.triangles.toLocaleString('pt-BR')} triângulos e ${op.summary_nodes.toLocaleString('pt-BR')} resumos. Fronteiras de pixel têm prioridade sobre a histerese.`;
$('measurements').innerHTML='<table><tr><th>Método</th><th>RMSE filtrado</th><th>Erro de cobertura</th><th>GPU nativa ms</th><th>p10–p90 ms</th></tr>'+r.methods.map(m=>`<tr><td>${names[m.method]}</td><td>${fmt(m.filtered_error.rmse_surface)}</td><td>${fmt(m.filtered_error.alpha_rmse)}</td><td>${fmt(m.native_timing.gpu_ms_median,4)}</td><td>${fmt(m.native_timing.gpu_ms_p10,4)}–${fmt(m.native_timing.gpu_ms_p90,4)}</td></tr>`).join('')+'</table>';
$('cpu').textContent=`Seleção CPU: ${fmt(r.selection_cpu_us,1)} µs. Validação e envio de índices à fila: ${fmt(r.index_validation_and_queue_us,1)} µs, sem aguardar conclusão GPU. Índices enviados por pose: ${(r.index_upload_bytes/1024).toFixed(1)} KiB. Essas medidas individuais são ruidosas.`;
const W=1000,H=250,L=65,R=25,T=15,B=35,X=d=>L+Math.log(d/2.5)/Math.log(64/2.5)*(W-L-R),Y=t=>T+(1-t/65536)*(H-T-B);let svg=`<svg viewBox="0 0 ${W} ${H}" role="img" aria-label="Triângulos por distância em escala logarítmica">`;for(const y of [0,32768,65536])svg+=`<line x1="${L}" x2="${W-R}" y1="${Y(y)}" y2="${Y(y)}" stroke="#35465a"/><text x="0" y="${Y(y)+4}" fill="#b6c5d5" font-size="12">${y}</text>`;
for(const [begin,end,color,field] of [[0,data.turn_frame+1,'#ffad57','triangles'],[data.turn_frame,rows.length,'#62beff','triangles'],[0,data.turn_frame+1,'#8994a3','previous_triangles']])svg+=`<polyline points="${rows.slice(begin,end).map(q=>X(q.parameters.distance)+','+Y(q[field])).join(' ')}" fill="none" stroke="${color}" stroke-width="2"/>`;svg+=`<circle cx="${X(r.parameters.distance)}" cy="${Y(r.triangles)}" r="5" fill="white"/>`;for(const d of [2.5,8,16,32,64])svg+=`<text x="${X(d)}" y="${H-8}" fill="#b6c5d5" font-size="12" text-anchor="middle">${d}</text>`;$('chart').innerHTML=svg+'</svg><p>Laranja: multiescala afastando · azul: multiescala aproximando · cinza: método anterior afastando. Eixo de distância logarítmico.</p>';
}
function step(delta){$('frame').value=(Number($('frame').value)+delta+data.frames_per_case)%data.frames_per_case;show()}
for(const id of ['scene','zoom','pixels','errors','control'])$(id).onchange=()=>{stop();show()};$('frame').oninput=()=>{stop();show()};$('back').onclick=()=>{stop();step(-1)};$('next').onclick=()=>{stop();step(1)};$('play').onclick=()=>{if(timer)stop();else{$('play').textContent='Pausar';timer=setInterval(()=>step(1),400)}};
const memory=data.memory;$('memory').textContent=`Árvore: ${data.tree_nodes.toLocaleString('pt-BR')} nós, preparação ${data.tree_build_ms.toFixed(2)} ms. Distribuições: ${(memory.distribution_bytes/1048576).toFixed(2)} MiB; proxies: ${(memory.proxy_vertex_bytes/1048576).toFixed(2)} MiB; metadados CPU: ${(memory.node_metadata_bytes/1048576).toFixed(2)} MiB; geometria: ${(memory.full_vertex_bytes/1048576).toFixed(2)} MiB; capacidade dos índices GPU: ${(memory.gpu_index_capacity_bytes/1048576).toFixed(2)} MiB. Recursos de controle, targets e memória interna do driver são adicionais.`;
$('metadata').textContent=`${data.adapter} · ${data.cases.length} frames · ${data.total_seconds.toFixed(1)} s de execução offline · teste de laboratório, sem integração no renderer da engine.`;show();
</script></html>'''
if (root.parent / 'quality' / 'report.json').is_file():
    page = page.replace('<h1>', '<p class="note"><a href="../quality/index.html">Abrir comparação nova: qualidade configurável e transição gradual →</a></p><h1>', 1)
(root/'index.html').write_text(page.replace('__DATA__',json.dumps(data,ensure_ascii=False).replace('<','\\u003c')))
print(root/'index.html')
try:
    from PIL import Image,ImageDraw,ImageFont
except ImportError:
    pass
else:
    canvas=Image.new('RGB',(1280,1090),'#101722');draw=ImageDraw.Draw(canvas)
    try:
        font=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',18)
        small=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',15)
        title=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',26)
    except OSError:
        font=small=title=ImageFont.load_default()
    draw.text((24,18),'IGNIS / resumos espaciais multiescala',font=title,fill='#e8eef7')
    draw.text((24,58),'Mesmo recorte ampliado por linha. Os pequenos blocos são pixels reais da imagem.',font=small,fill='#b6c5d5')
    for row,frame in enumerate([0,14,24]):
        r=next(r for r in data['cases'] if r['case']=='glossy' and r['frame']==frame)
        b=r['screen_bounds'];side=min(256,int(max(b[2]-b[0],b[3]-b[1]))+7)
        x=max(0,min(256-side,int((b[0]+b[2]-side)/2)));y=max(0,min(256-side,int((b[1]+b[3]-side)/2)))
        yy=105+row*325
        draw.text((24,yy),f'Distância {r["parameters"]["distance"]:.2f} / multiescala {r["triangles"]:,} triângulos / anterior {r["previous_triangles"]:,}',font=font,fill='#ffad57')
        for col,(method,label) in enumerate([('reference','Referência integrada'),('multiscale','Multiescala'),('previous','Seleção anterior'),('fixed','Sempre resumo fixo')]):
            img=Image.open(root/(r['id']+'_'+method+'.png')).crop((x,y,x+side,y+side)).resize((250,250),Image.Resampling.NEAREST)
            xx=24+col*312;canvas.paste(img,(xx+20,yy+35));draw.text((xx,yy+291),label,font=small,fill='#b6c5d5')
    canvas.save(root/'comparison.png')
