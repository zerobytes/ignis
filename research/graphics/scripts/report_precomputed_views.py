#!/usr/bin/env python3
"""Local comparison of persisted view/light tables, geometry, and reprojection."""
import csv
import json
from pathlib import Path
from statistics import median
ROOT = Path('artifacts/appearance-probe/views')
data = json.loads((ROOT / 'report.json').read_text())
rows = []
for case in data['cases']:
    for index, method in enumerate(case['methods']):
        timing = case['timings'][[1, 2, 3][index]]
        rows.append(dict(scene=case['scene'], frame=case['frame'], method=index,
                         camera=case['pose']['camera'], light=case['pose']['light'],
                         distance=case['pose']['distance'], rmse=method['error']['rmse_surface'],
                         alpha_rmse=method['error']['alpha_rmse'], temporal=method['temporal_residual'],
                         gpu_ms=timing['gpu']['median'], full_gpu_ms=case['timings'][0]['gpu']['median'],
                         serial_frame_ms=timing['serial_frame']['median'],
                         full_serial_frame_ms=case['timings'][0]['serial_frame']['median']))
with (ROOT / 'results.csv').open('w', newline='') as f:
    writer = csv.DictWriter(f, fieldnames=list(rows[0]))
    writer.writeheader(); writer.writerows(rows)
summary = {}
for scene in dict.fromkeys(c['scene'] for c in data['cases']):
    summary[scene] = []
    for method in range(3):
        selected = [r for r in rows if r['scene'] == scene and r['method'] == method]
        summary[scene].append(dict(rmse_median=median(r['rmse'] for r in selected),
            rmse_max=max(r['rmse'] for r in selected),
            alpha_median=median(r['alpha_rmse'] for r in selected),
            gpu_ratio=median(r['gpu_ms']/r['full_gpu_ms'] for r in selected),
            serial_frame_ratio=median(r['serial_frame_ms']/r['full_serial_frame_ms'] for r in selected)))
(ROOT/'summary.json').write_text(json.dumps(summary, indent=2))
page = r'''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Ignis · Vistas preparadas no build</title>
<style>:root{color-scheme:dark;font-family:system-ui,sans-serif;background:#0c121b;color:#e9eef6}body{max-width:1400px;margin:auto;padding:30px 24px}h1{font-size:38px;letter-spacing:-1px;margin:12px 0}h2{font-size:22px}p{color:#b7c6d8;line-height:1.65;max-width:1200px}.tag{color:#ffb66b;font-size:12px;letter-spacing:2px}.box{background:#162130;border:1px solid #34465e;border-radius:12px;padding:20px;margin:20px 0}.controls{display:flex;gap:16px;align-items:center;flex-wrap:wrap}.controls+.controls{margin-top:20px}input[type=range]{flex:1;min-width:230px;accent-color:#ffb66b}button,select{font:inherit;color:inherit;background:#263951;border:1px solid #49637f;border-radius:6px;padding:8px}.grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:18px}figure{margin:0}canvas{width:100%;image-rendering:pixelated;border:1px solid #34465e}figcaption{font-size:14px;line-height:1.5;margin-top:10px}.stats{display:flex;gap:32px;flex-wrap:wrap}.stats b{font-size:25px;display:block;color:#fff}.stats span{font-size:13px;color:#b7c6d8}table{width:100%;border-collapse:collapse;font-size:13px}th,td{padding:9px;text-align:right;border-bottom:1px solid #34465e}th:first-child,td:first-child{text-align:left}.scroll{overflow:auto}.note{border-left:3px solid #ffb66b;padding-left:16px}a{color:#ffb66b}summary{cursor:pointer}svg{width:100%}.small{font-size:12px} @media(max-width:900px){.grid{grid-template-columns:1fr 1fr}h1{font-size:28px}} @media(max-width:480px){.grid{grid-template-columns:1fr}}</style>
<div class="tag">IGNIS / PESQUISA GRÁFICA / EAF-69</div><h1>Preparar vistas. Consultar durante o jogo.</h1>
<p>Órbita e luz móveis usando tabelas HDR geradas em uma etapa separada e carregadas do disco. O teste compara a geometria completa com interpolação de respostas de material e reposicionamento das vistas por altura. Os ângulos principais não foram usados no bake.</p>
<div class="box"><div class="controls"><label>Percurso <select id="scene"><option value="both">Câmera + luz</option><option value="camera">Só câmera</option><option value="light">Só luz</option><option value="near">Próximo · câmera + luz</option><option value="seam">Passagem 360° → 0°</option></select></label><label>Orçamento <select id="budget"><option value="1">24 vistas × 24 luzes</option><option value="0">12 vistas × 12 luzes</option></select></label><button id="play">Reproduzir</button><button id="back" aria-label="Frame anterior">←</button><button id="next" aria-label="Próximo frame">→</button><label><input type="checkbox" id="zoom" checked> Ampliar região central</label></div>
<div class="controls"><label for="frame">Posição</label><input type="range" id="frame" min="0" value="0"><span id="position"></span></div><p id="pose"></p><div id="stats" class="stats"></div></div>
<div class="grid" id="images"></div><p class="note" id="quality"></p>
<div class="box"><h2>Custo na posição atual</h2><div id="timings" class="scroll"></div><p>GPU inclui o desenho e as consultas à tabela, sem reconstruir o asset. “Frame serial” mede atualização dos parâmetros, codificação, submissão e espera pela GPU, sem leitura de imagem; representa este laboratório bloqueante, não FPS do gameplay. As imagens são pré-calculadas para inspeção; os tempos vêm da execução nativa dos shaders.</p></div>
<div class="box"><h2>Memória e preparação</h2><div id="memory" class="scroll"></div><p>Valores de payload residente por representação. As tabelas já incluem cores HDR e altura/cobertura; não exigem a malha original para desenhar o proxy. O laboratório mantém a malha para a referência e as duas tabelas para comparação. Buffers de captura supersampled e estruturas do driver são custos adicionais da instrumentação. Não há compressão do atlas nem mipmaps nesta versão.</p></div>
<div class="box"><h2>Ganho e erro ao longo do percurso</h2><div id="chart"></div><p id="aggregate"></p></div>
<details class="box"><summary><b>O que este experimento comprova — e seus limites</b></summary><p>A câmera gira em azimute com elevação55°; luz direcional gira a35°. Material misto com resposta difusa e brilho Blinn–Phong, luz unitária, sem sombras projetadas ou GI. As respostas angulares são calculadas com a mesma função usada na geometria completa. A tabela contém cores já iluminadas para essas condições; novos materiais/elevações/animações exigem novo preparo ou outra representação.</p><p>O proxy desenha dois triângulos sobre uma região conservadora do objeto;639 comparações nativas confirmam pixels idênticos à consulta sobre a tela inteira. Os raios usam coordenadas absolutas do fragmento, evitando dependência da triangulação do proxy. O atlas guarda somente a altura da superfície visível por vista. Três iterações reposicionam amostras no raio da câmera atual. Ponte, bordas, buracos e partes recém-expostas podem falhar: mistura de altura/cobertura não representa múltiplas camadas. O controle “plano” usa a mesma tabela24×24 sem essa correção. Este teste não integra seleção hierárquica adaptativa nem decide quando retornar à geometria.</p><p>Bake128² por vista/resposta com16 amostras espaciais; referência256 amostras por pixel, candidatos e controle de geometria16. Timestamps de todos os métodos em256² com uma amostra;63 repetições intercaladas após aquecimento, p10–p90 visível. Não transformar redução de triângulos ou tempo de shader em ganho equivalente de FPS. Erro temporal é diferença de resíduos em coordenadas de tela, sem alinhamento por movimento.</p><p>Artefatos versionados e assinados por geometria, shader e configuração; checksum rejeita corrupção/truncamento. O comando de teste não faz bake escondido. A interface reproduz as capturas e não manipula a demo da engine. Integração futura deverá compartilhar comandos entre MCP e UI.</p></details>
<p><a href="report.json">Dados e controles</a> · <a href="results.csv">CSV</a> · <a href="summary.json">Resumo numérico</a> · <a href="../cost/index.html">Custo dos resumos anteriores</a></p><p class="small" id="metadata"></p>
<script>
const data=__DATA__,summary=__SUMMARY__,$=id=>document.getElementById(id),fmt=(x,n=4)=>x==null?'indisponível':x.toFixed(n),series=()=>data.cases.filter(c=>c.scene===$('scene').value);let timer=null,request=0;const cache=new Map();
function stop(){clearInterval(timer);timer=null;$('play').textContent='Reproduzir'}
function load(file){if(cache.has(file))return cache.get(file);const value=new Promise((resolve,reject)=>{const img=new Image();img.onload=()=>resolve(img);img.onerror=()=>{cache.delete(file);reject(Error('Imagem indisponível: '+file))};img.src=file});cache.set(file,value);while(cache.size>48)cache.delete(cache.keys().next().value);return value}
function canvas(image,row){const c=document.createElement('canvas');c.width=c.height=256;const ctx=c.getContext('2d');ctx.imageSmoothingEnabled=false;const w=$('zoom').checked?(row.pose.distance<5?210:85):256,x=(256-w)/2;ctx.drawImage(image,x,x,w,w,0,0,256,256);return c}
async function show(){const ticket=++request,rows=series(),i=Math.min(Number($('frame').value),rows.length-1),r=rows[i],m=Number($('budget').value);$('frame').max=rows.length-1;
try{const figs=await Promise.all([['reference','Referência integrada · geometria completa'],['m'+m,'Vistas + resposta pré-calculada · com altura'],['m2','Controle24×24 · mistura sobre plano'],['m'+m+'_error','Erro contra referência ×8']].map(async([suffix,title])=>{const img=await load(r.id+'_'+suffix+'.png'),f=document.createElement('figure');f.append(canvas(img,r));const caption=document.createElement('figcaption');caption.textContent=title;f.append(caption);return f}));if(ticket!==request)return;$('images').replaceChildren(...figs)}catch(e){if(ticket===request)$('pose').textContent=e.message;return}
$('position').textContent=`${i+1}/${rows.length}`;$('pose').textContent=`Câmera ${fmt(r.pose.camera,2)}° · luz ${fmt(r.pose.light,2)}° · distância ${r.pose.distance}`;const err=r.methods[m].error,t=r.timings[m+1],full=r.timings[0];
$('stats').innerHTML=[[fmt(err.rmse_surface),'RMSE RGB linear'],[fmt(t.gpu.median)+'ms','GPU · tabela'],[fmt(full.gpu.median)+'ms','GPU · geometria completa'],[fmt(t.gpu.median/full.gpu.median,2)+'×','razão de custo GPU']].map(([a,b])=>`<span><b>${a}</b>${b}</span>`).join('');
$('quality').textContent=`Erro de cobertura: ${fmt(err.alpha_rmse)}. Erro RGB máximo: ${fmt(err.max_abs)}. Resíduo temporal: ${fmt(r.methods[m].temporal_residual)}. O controle completo com16amostras tem RMSE ${fmt(r.full_error.rmse_surface)} contra a referência256amostras; o restante pode incluir filtragem espacial, interpolação angular e reconstrução de visibilidade.`;
$('timings').innerHTML='<table><tr><th>Método</th><th>GPU ms</th><th>p10–p90</th><th>Frame serial ms</th><th>Parâmetros/envio µs</th></tr>'+['Geometria completa','12×12 · altura','24×24 · altura','24×24 · plano'].map((name,j)=>{const t=r.timings[j];return `<tr><td>${name}</td><td>${fmt(t.gpu.median)}</td><td>${fmt(t.gpu.p10)}–${fmt(t.gpu.p90)}</td><td>${fmt(t.serial_frame.median)}</td><td>${fmt(t.selection_uniform_us,2)}</td></tr>`}).join('')+'</table>';
const a=summary[r.scene][m];$('aggregate').textContent=`Percurso atual: mediana das razões GPU tabela/geometria = ${fmt(a.gpu_ratio,2)}×; frame serial = ${fmt(a.serial_frame_ratio,2)}×. RMSE mediano ${fmt(a.rmse_median)}, pior ${fmt(a.rmse_max)}. Menor que1 no custo significa economia; erro mais próximo de0 significa maior fidelidade.`;
const W=1000,H=220,L=55,R=20,T=15,B=25,X=j=>L+j/(rows.length-1)*(W-L-R),Y=v=>T+(1-Math.min(v,2)/2)*(H-T-B);let svg=`<svg viewBox="0 0 ${W} ${H}" role="img" aria-label="Razão de tempo GPU das tabelas sobre geometria completa">`;for(const v of [0,1,2])svg+=`<line x1="${L}" x2="${W-R}" y1="${Y(v)}" y2="${Y(v)}" stroke="${v===1?'#dbe6f2':'#34465e'}"/><text x="5" y="${Y(v)+4}" fill="#b7c6d8" font-size="12">${v}×</text>`;for(const [j,color] of [[1,'#ffb66b'],[2,'#61c8ee']])svg+=`<polyline points="${rows.map((c,k)=>X(k)+','+Y(c.timings[j].gpu.median/c.timings[0].gpu.median)).join(' ')}" stroke="${color}" fill="none" stroke-width="2"/>`;svg+=`<line x1="${X(i)}" x2="${X(i)}" y1="${T}" y2="${H-B}" stroke="white"/><text x="65" y="218" fill="#ffb66b" font-size="12">12×12</text><text x="145" y="218" fill="#61c8ee" font-size="12">24×24 · escala do gráfico limitada a2×</text></svg>`;$('chart').innerHTML=svg;
}
function step(d){$('frame').value=(Number($('frame').value)+d+series().length)%series().length;show()}
$('frame').oninput=()=>{stop();show()};for(const id of ['scene','budget','zoom'])$(id).onchange=()=>{stop();show()};$('back').onclick=()=>{stop();step(-1)};$('next').onclick=()=>{stop();step(1)};$('play').onclick=()=>{if(timer)stop();else{$('play').textContent='Pausar';timer=setInterval(()=>step(1),180)}};
$('memory').innerHTML='<table><tr><th>Representação</th><th>Payload MiB</th><th>Vezes geometria</th><th>Bake s</th><th>Leitura/validação/upload ms</th></tr>'+`<tr><td>Geometria · ${data.triangles.toLocaleString('pt-BR')}triângulos</td><td>${fmt(data.geometry_bytes/1048576,2)}</td><td>1,00×</td><td>—</td><td>—</td></tr>`+data.atlases.map(a=>`<tr><td>${a.manifest.count}vistas ×${a.manifest.count}luzes</td><td>${fmt(a.manifest.bytes/1048576,2)}</td><td>${fmt(a.manifest.bytes/data.geometry_bytes,2)}×</td><td>${fmt(a.manifest.bake_seconds,2)}</td><td>${fmt(a.load_ms,2)}</td></tr>`).join('')+'</table>';
$('metadata').textContent=`${data.adapter} · ${data.cases.length}poses · bake e teste em processos separados · aceite visual e navegador nativo separados da validação automática.`;show();
</script></html>'''
if (ROOT.parent/'compression'/'index.html').exists():
    page=page.replace('<h1>', '<p class="note"><a href="../compression/index.html">Novo: compactação do atlas com comparação direta →</a></p><h1>',1)
(ROOT/'index.html').write_text(page.replace('__DATA__',json.dumps(data,ensure_ascii=False).replace('<','\\u003c')).replace('__SUMMARY__',json.dumps(summary)))
print(ROOT/'index.html')
try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    pass
else:
    image=Image.new('RGB',(1280,1090),'#0c121b');draw=ImageDraw.Draw(image)
    font=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',18)
    draw.text((24,20),'IGNIS / vistas e respostas de material preparadas no build',font=font,fill='white')
    for row,id in enumerate(['both_06','both_24','near_12']):
        case=next(c for c in data['cases'] if c['id']==id);y=75+row*330
        draw.text((24,y),f"{id} · camera {case['pose']['camera']:.2f}° · luz {case['pose']['light']:.2f}°",font=font,fill='#ffb66b')
        for col,(suffix,label) in enumerate([('reference','Referencia'),('m1','24x24 com altura'),('m2','24x24 sobre plano'),('m1_error','Erro x8')]):
            source=Image.open(ROOT/f'{id}_{suffix}.png');w=210 if case['pose']['distance']<5 else 85;x=(256-w)//2
            source=source.crop((x,x,x+w,x+w)).resize((250,250),Image.Resampling.NEAREST);image.paste(source,(24+col*312,y+32));draw.text((24+col*312,y+292),label,font=font,fill='white')
    image.save(ROOT/'comparison.png')
