"""Report incremental performance; never add speedups from separate experiments."""
import json,statistics,html
from pathlib import Path
R=Path('artifacts/appearance-probe/combined-symbol')
def read(n):return json.loads((R/f'{n}.json').read_text())
def med(s,k):return statistics.median(s[k])
def pair(r,m,k='total'):
    return statistics.mean(med(s,k) for s in r['samples'] if s['mode']==m)
summary=[]
for phase in ['timings','repeat']:
    for r in read(phase):
        b=pair(r,135);f=pair(r,305);c=pair(r,315)
        summary.append(dict(phase=phase,density=r['density'],factor=[1,2,10,100][r['density']],resolution=r['resolution'],distance=r['distance'],baseline_ms=b,forced_ms=f,optional_ms=c,forced_ratio=f/b,optional_ratio=c/b,baseline_gpu_ms=pair(r,135,'gpu'),forced_gpu_ms=pair(r,305,'gpu'),optional_gpu_ms=pair(r,315,'gpu'),selected=r['selected']))
quality=read('quality')
checks=[dict(path=n,passed=sum(r['checks'][i]['quality_gate'] for r in quality),max_rmse=max(r['checks'][i]['foreground_rmse'] for r in quality),max_p99=max(r['checks'][i]['p99_rgb'] for r in quality),max_coverage=max(r['checks'][i]['coverage_fraction'] for r in quality),max_temporal=max(r['checks'][i]['temporal_residual_rmse'] or 0 for r in quality)) for i,n in enumerate(['anterior','forçado','opcional'])]
spreads=[]
for phase in ['timings','repeat']:
 for r in read(phase):
  for mode in [135,305,315]:
   ds=[med(s,'total') for s in r['samples'] if s['mode']==mode]
   spreads.append(ds[1]/ds[0])
result=dict(quality=checks,poses=len(quality),selected=sum(r['symbol_selected'] for r in quality),max_delta_rgb=max(r['forced_vs_baseline']['max_rgb'] for r in quality),max_delta_rmse=max(r['forced_vs_baseline']['foreground_rmse'] for r in quality),duplicate_ratio_range=[min(spreads),max(spreads)],rows=summary)
(R/'analysis.json').write_text(json.dumps(result,indent=2)+'\n')
rows=''.join(f"<tr><td>{r['phase']}</td><td>{r['factor']}×</td><td>{r['resolution']}</td><td>{r['distance']}</td><td>{r['baseline_ms']:.4f}</td><td>{r['forced_ms']:.4f}</td><td>{r['optional_ms']:.4f}</td><td>{(r['forced_ratio']-1)*100:+.1f}%</td></tr>" for r in summary)
data=json.dumps(quality,separators=(',',':'))
page='''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Ignis · símbolos opcionais</title>
<style>body{background:#10151b;color:#e5ebef;font:15px system-ui;margin:24px}h1{font-size:25px}p{max-width:1100px;line-height:1.55}button,select,input{padding:8px;background:#253241;color:inherit;border:1px solid #607182;border-radius:5px}label{margin-right:12px;display:inline-block}main{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px}figure{margin:0}img{width:100%;cursor:crosshair}canvas{border:1px solid #607182;max-width:100%}small{display:block;color:#b6c4cf}table{border-collapse:collapse;margin-top:20px}td,th{padding:8px;border-bottom:1px solid #334252;text-align:right}.notice{background:#263548;padding:14px;border-radius:6px}#metrics{white-space:pre-wrap;line-height:1.5}@media(max-width:800px){main{grid-template-columns:1fr}}</style>
<h1>Anterior versus anterior + símbolo opcional</h1>
<p class="notice"><b>Escopo:</b> seleção global/regional + automático forward/depth + geometria compactada (135), com resposta de material binária opcional. A forma e os triângulos selecionados são os mesmos. Este é um símbolo de material Blinn-Phong; o atlas geométrico da pedra continua reprovado e não participa. POLICY_RESULT</p>
<p>Resolução física de origem, não CSS. 1× a 100× são os assets do estudo anterior, com frequências de relevo diferentes; não representam triângulos por pixel nem a mesma superfície subdividida. Imagens abaixo são capturas, e não renderização/medição ao vivo no navegador. Não há comprovação de aceite visual humano.</p>
<div><label>Densidade <select id="density"></select></label><label>Resolução <select id="resolution"></select></label><label>Distância <select id="distance"></select></label><label>Movimento <select id="scene"></select></label><label>Candidato <select id="candidate"><option value="auto">Opcional</option><option value="forced">Forçado (diagnóstico)</option></select></label></div>
<p><button id="play">Reproduzir capturas</button> <input id="frame" type="range" min="0" max="8" step="1" value="0"><span id="position"></span></p>
<main><figure><figcaption>A · geometria completa</figcaption><img id="a"><canvas id="ca"></canvas></figure><figure><figcaption>B · melhor anterior congelado</figcaption><img id="b"><canvas id="cb"></canvas></figure><figure><figcaption>C/D · símbolo opcional / forçado</figcaption><img id="c"><canvas id="cc"></canvas></figure></main>
<p><small>Imagens gerais ajustadas à largura. Clique para posicionar os recortes: neles, 1 pixel renderizado = 1 pixel físico do canvas (DPR atual). O playback tem nove amostras por percurso; não prova fluidez temporal contínua.</small></p><div id="metrics"></div>
<details><summary>Tempos completos e repetição (ms; negativo = candidato mais barato)</summary><p>GPU e tempo total são medições distintas. A tabela apresenta throughput CPU+GPU em blocos de64frames, oito rodadas, dois controles por caminho, câmera/luz atualizadas por frame. Variação de controles não é ganho.</p><table><thead><tr><th>Rodada</th><th>Fonte</th><th>Res.</th><th>Dist.</th><th>Anterior</th><th>Forçado</th><th>Opcional</th><th>Forçado/anterior</th></tr></thead><tbody>ROWS</tbody></table></details>
<script>const DATA=DATA_PLACEHOLDER;
const $=id=>document.getElementById(id);let generation=0,point=[.5,.5],timer=null;
function options(id,values,labels){$(id).replaceChildren(...values.map((v,i)=>{const o=document.createElement('option');o.value=v;o.textContent=labels[i];return o}));}
options('density',[0,1,2,3],['1×','2×','10×','100×']);options('resolution',[0,1],['3456 × 2234','1728 × 1117']);options('distance',[14,28,224],['Perto · 14','Médio · 28','Longe · 224']);
function scenes(){const old=$('scene').value;const extra=+$('density').value===3&&+$('distance').value===14;options('scene',extra?[2,0,1]:[2],extra?['Câmera + luz','Só câmera','Só luz']:['Câmera + luz']);if([...$('scene').options].some(o=>o.value===old))$('scene').value=old;}
function current(){return DATA.find(r=>r.density===+$('density').value&&r.resolution===+$('resolution').value&&r.distance===+$('distance').value&&r.scene===+$('scene').value&&r.frame===+$('frame').value);}
function crop(){for(const name of ['a','b','c']){const im=$(name),c=$('c'+name),dpr=window.devicePixelRatio||1;const w=Math.min(360,Math.max(100,im.clientWidth));c.width=Math.round(w*dpr);c.height=Math.round(180*dpr);c.style.width=w+'px';c.style.height='180px';const x=Math.max(0,Math.min(im.naturalWidth-c.width,Math.round(point[0]*im.naturalWidth-c.width/2))),y=Math.max(0,Math.min(im.naturalHeight-c.height,Math.round(point[1]*im.naturalHeight-c.height/2)));const ctx=c.getContext('2d');ctx.imageSmoothingEnabled=false;ctx.clearRect(0,0,c.width,c.height);ctx.drawImage(im,x,y,c.width,c.height,0,0,c.width,c.height);}}
async function render(){const r=current();if(!r)return;const g=++generation;const forced=$('candidate').value==='forced';const mode=forced||r.symbol_selected?305:135;const files=[0,135,mode];await Promise.all(['a','b','c'].map((id,i)=>new Promise(resolve=>{const im=$(id);im.onload=resolve;im.onerror=resolve;im.src=r.id+'_'+files[i]+'.png';if(im.complete)resolve();})));if(g!==generation)return;crop();$('position').textContent=' amostra '+r.frame+'/8';const q=r.checks[forced?1:2];$('metrics').textContent=`${r.size.join(' × ')} · ${r.triangles.toLocaleString()} triângulos selecionados / ${r.source_triangles.toLocaleString()} fonte\nEscolha automática: ${r.symbol_selected?'símbolo':'anterior (fallback)'} · candidato exibido: ${forced?'forçado':'opcional'}\nRMSE ${q.foreground_rmse.toFixed(6)} · p99 ${q.p99_rgb.toFixed(6)} · divergência de cobertura ${q.coverage_mismatches} pixels · critério ${q.quality_gate?'PASSOU':'FALHOU'}`;}
for(const id of ['density','resolution','distance','scene','candidate'])$(id).onchange=()=>{if(id==='density'||id==='distance')scenes();render()};$('frame').oninput=render;
for(const id of ['a','b','c'])$(id).onclick=e=>{const rect=$(id).getBoundingClientRect();point=[(e.clientX-rect.left)/rect.width,(e.clientY-rect.top)/rect.height];crop()};
$('play').onclick=()=>{if(timer){clearInterval(timer);timer=null;$('play').textContent='Reproduzir capturas'}else{timer=setInterval(()=>{$('frame').value=(+$('frame').value+1)%9;render()},650);$('play').textContent='Pausar'}};window.onresize=crop;scenes();render();
</script></html>'''.replace('ROWS',rows).replace('DATA_PLACEHOLDER',data).replace('POLICY_RESULT', 'A calibração não encontrou benefício robusto: política conservou o anterior em todos os casos.' if not read('fit')['enabled_mask'] else 'A calibração habilitou classes de trabalho; a seleção registrada abaixo é congelada antes da validação.')
(R/'index.html').write_text(page)
print(json.dumps({k:v for k,v in result.items() if k!='rows'},indent=2))
