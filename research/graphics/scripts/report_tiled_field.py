"""Report incremental performance; never add speedups from separate experiments."""
import json,statistics,html
from pathlib import Path
R=Path('artifacts/appearance-probe/tiled-field')
def read(n):return json.loads((R/f'{n}.json').read_text())
def med(s,k):return statistics.median(s[k])
def pair(r,m,k='total'):
    return statistics.mean(med(s,k) for s in r['samples'] if s['mode']==m)
summary=[]
for phase in ['timings','repeat']:
    for r in read(phase):
        b=pair(r,135);f=pair(r,395);c=pair(r,405)
        summary.append(dict(phase=phase,density=r['density'],factor=[1,2,10,100][r['density']],resolution=r['resolution'],distance=r['distance'],baseline_ms=b,forced_ms=f,optional_ms=c,forced_ratio=f/b,optional_ratio=c/b,hierarchy_ms=pair(r,345),tiles_vs_hierarchy=f/pair(r,345),baseline_gpu_ms=pair(r,135,'gpu'),forced_gpu_ms=pair(r,395,'gpu'),optional_gpu_ms=pair(r,405,'gpu'),eligible=r['eligible'],baseline_cpu_submit_ms=pair(r,135,'cpu_submit'),forced_cpu_submit_ms=pair(r,395,'cpu_submit'),optional_cpu_submit_ms=pair(r,405,'cpu_submit'),selected=r['selected']))
quality=read('quality')
checks=[dict(path=n,passed=sum(r['checks'][i]['quality_gate'] for r in quality),max_rmse=max(r['checks'][i]['foreground_rmse'] for r in quality),max_p99=max(r['checks'][i]['p99_rgb'] for r in quality),max_coverage=max(r['checks'][i]['coverage_fraction'] for r in quality),max_temporal=max(r['checks'][i]['temporal_residual_rmse'] or 0 for r in quality)) for i,n in enumerate(['anterior','forçado','opcional'])]
audit=[]
for r in read('timestamp-audit'):
 for m in [135,395,405]:
  audit.append(dict(density=r['density'],resolution=r['resolution'],mode=m,total_ms=pair(r,m),gpu_envelope_ms=pair(r,m,'gpu'),pass_latency_ms=pair(r,m,'pass_latency'),instrumented_wall_ms=pair(r,m,'instrumented_total'),max_overlap=max(max(s['gpu_overlap_count']) for s in r['samples'] if s['mode']==m)))
spreads=[]
for phase in ['timings','repeat']:
 for r in read(phase):
  for mode in [135,395,405]:
   ds=[med(s,'total') for s in r['samples'] if s['mode']==mode]
   spreads.append(ds[1]/ds[0])
result=dict(timestamp_audit=audit,quality=checks,poses=len(quality),selected=sum(r['symbol_selected'] for r in quality),eligible=sum(r['eligible'] for r in quality),max_delta_rgb=max(r['forced_vs_baseline']['max_rgb'] for r in quality),max_delta_rmse=max(r['forced_vs_baseline']['foreground_rmse'] for r in quality),duplicate_ratio_range=[min(spreads),max(spreads)],rows=summary)
result['fixed_quality']=read('fixed-quality');result['fixed_timings']=read('fixed-timings');result['fixed_preparation']=read('fixed-preparation')
(R/'analysis.json').write_text(json.dumps(result,indent=2)+'\n')
rows=''.join(f"<tr><td>{r['phase']}</td><td>{r['factor']}×</td><td>{r['resolution']}</td><td>{r['distance']}</td><td>{r['baseline_ms']:.4f}</td><td>{r['hierarchy_ms']:.4f}</td><td>{r['forced_ms']:.4f}</td><td>{r['optional_ms']:.4f}</td><td>{(r['forced_ratio']-1)*100:+.1f}%</td><td>{(r['tiles_vs_hierarchy']-1)*100:+.1f}%</td></tr>" for r in summary)
fixed_summary=[]
for r in read('fixed-timings'):
 values={m:pair(r,m) for m in [1140,135,345,395,405]}
 fixed_summary.append(dict(resolution=r['resolution'],target=r['target'],distance=r['distance'],dense_ms=values[1140],baseline_ms=values[135],hierarchy_ms=values[345],tiles_ms=values[395],optional_ms=values[405]))
result['fixed_summary']=fixed_summary
(R/'analysis.json').write_text(json.dumps(result,indent=2)+'\n')
fixed_rows=''.join(f"<tr><td>{r['target']}</td><td>{r['resolution']}</td><td>{r['distance']}</td><td>{r['dense_ms']:.4f}</td><td>{r['baseline_ms']:.4f}</td><td>{r['hierarchy_ms']:.4f}</td><td>{r['tiles_ms']:.4f}</td><td>{r['optional_ms']:.4f}</td></tr>" for r in fixed_summary)
data=json.dumps(quality,separators=(',',':'))
page='''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Ignis · busca compartilhada por blocos</title>
<style>body{background:#10151b;color:#e5ebef;font:15px system-ui;margin:24px}h1{font-size:25px}p{max-width:1100px;line-height:1.55}button,select,input{padding:8px;background:#253241;color:inherit;border:1px solid #607182;border-radius:5px}label{margin-right:12px;display:inline-block}main{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px}figure{margin:0}img{width:100%;cursor:crosshair}canvas{border:1px solid #607182;max-width:100%}small{display:block;color:#b6c4cf}table{border-collapse:collapse;margin-top:20px}td,th{padding:8px;border-bottom:1px solid #334252;text-align:right}.notice{background:#263548;padding:14px;border-radius:6px}#metrics{white-space:pre-wrap;line-height:1.5}@media(max-width:800px){main{grid-template-columns:1fr}}</style>
<h1>Busca compartilhada por blocos de pixels</h1>
<p class="notice"><b>Escopo:</b> seleção global/regional + automático forward/depth + geometria compactada (135), com símbolo geométrico com busca compartilhada por blocos. Um pré-passe delimita a busca por bloco 8×8; cada pixel resolve a superfície original. O certificado de 64 regiões preserva a geometria quando não consegue garantir uma única interseção. Domínio: heightfield, não uma pedra multicamada. Forçado também usa B em vistas não certificadas. POLICY_RESULT</p>
<p>Resolução física de origem, não CSS. 1× a 100× são os assets do estudo anterior, com frequências de relevo diferentes; não representam triângulos por pixel nem a mesma superfície subdividida. Imagens abaixo são capturas, e não renderização/medição ao vivo no navegador. Não há comprovação de aceite visual humano.</p>
<div><label>Densidade <select id="density"></select></label><label>Resolução <select id="resolution"></select></label><label>Distância <select id="distance"></select></label><label>Movimento <select id="scene"></select></label><label>Candidato <select id="candidate"><option value="auto">Opcional</option><option value="forced">Forçado (diagnóstico)</option></select></label></div>
<p><button id="play">Reproduzir capturas</button> <input id="frame" type="range" min="0" max="8" step="1" value="0"><span id="position"></span></p>
<main><figure><figcaption>A · geometria completa</figcaption><img id="a"><canvas id="ca"></canvas></figure><figure><figcaption>B · melhor anterior congelado</figcaption><img id="b"><canvas id="cb"></canvas></figure><figure><figcaption>C/T · símbolo opcional / blocos forçados</figcaption><img id="c"><canvas id="cc"></canvas></figure></main>
<p><small>Imagens gerais ajustadas à largura. Clique para posicionar os recortes: neles, 1 pixel renderizado = 1 pixel físico do canvas (DPR atual). O playback tem nove amostras por percurso; não prova fluidez temporal contínua.</small></p><div id="metrics"></div>
<p>Os blocos não aumentam o tamanho dos pixels. Além das alturas originais, a hierarquia guarda intervalos RG32Float em várias escalas. No asset 100×, a hierarquia acrescenta 0,667 MiB aos 6,260 MiB de alturas: 6,926 MiB de payload GPU. O scratch temporário por blocos reserva 967.680 B adicionais (0,923 MiB) para a resolução máxima. As normais são compartilhadas com B. O comparador conserva B para fallback: esta memória é adicional, não uma economia residente.</p>
<details><summary>Tempos completos e repetição (ms; negativo = candidato mais barato)</summary><p>GPU inclui compute, transição de recursos e draw: janela do primeiro início ao último fim de 32 frames, dividida por 32. Latências de passes sobrepostos não são somadas. CPU de submissão inclui driver/backpressure; não é tempo CPU puro. GPU e tempo total são medições distintas. A tabela apresenta throughput CPU+GPU em blocos de 64 frames, oito rodadas, dois controles por caminho, câmera/luz atualizadas por frame. Variação de controles não é ganho.</p><table><thead><tr><th>Rodada</th><th>Fonte</th><th>Res.</th><th>Dist.</th><th>Anterior</th><th>Hierarquia/pixel</th><th>Blocos</th><th>Opcional</th><th>Blocos/anterior</th><th>Blocos/hierarquia</th></tr></thead><tbody>ROWS</tbody></table></details>
<details><summary>Mesma superfície: 1, 2, 10 e 100 triângulos por pixel</summary><p>Controle separado com a parental fixa de 327.680 triângulos. Subdivisões densas reais, material e campo de normais constantes. Meta definida por pixels cobertos na pose inicial a14; muda durante o percurso. Todos os caminhos percorrem as mesmas quatro poses de câmera/luz: denso4 frames/bloco, otimizados32 (oito repetições), quatro rodadas e duas duplicatas. Tempos totais amortizados em ms; não comparação de latência individual. A escolha B/H/T/C é precedida do certificado da subdivisão e preserva a fonte densa se ele falhar.</p><table><thead><tr><th>Meta T/pixel</th><th>Res.</th><th>Dist.</th><th>Densa</th><th>B anterior</th><th>H por pixel</th><th>T blocos</th><th>C opcional</th></tr></thead><tbody>FIXED_ROWS</tbody></table></details>
<script>const DATA=DATA_PLACEHOLDER;
const $=id=>document.getElementById(id);let generation=0,point=[.5,.5],timer=null;
function options(id,values,labels){$(id).replaceChildren(...values.map((v,i)=>{const o=document.createElement('option');o.value=v;o.textContent=labels[i];return o}));}
options('density',[0,1,2,3],['1×','2×','10×','100×']);options('resolution',[0,1],['3456 × 2234','1728 × 1117']);options('distance',[14,28,224],['Perto · 14','Médio · 28','Longe · 224']);
function scenes(){const old=$('scene').value;const extra=+$('density').value===3&&[14,28].includes(+$('distance').value);options('scene',extra?[2,0,1]:[2],extra?['Câmera + luz','Só câmera','Só luz']:['Câmera + luz']);if([...$('scene').options].some(o=>o.value===old))$('scene').value=old;}
function current(){return DATA.find(r=>r.density===+$('density').value&&r.resolution===+$('resolution').value&&r.distance===+$('distance').value&&r.scene===+$('scene').value&&r.frame===+$('frame').value);}
function crop(){for(const name of ['a','b','c']){const im=$(name),c=$('c'+name),dpr=window.devicePixelRatio||1;const w=Math.min(360,Math.max(100,im.clientWidth));c.width=Math.round(w*dpr);c.height=Math.round(180*dpr);c.style.width=w+'px';c.style.height='180px';const x=Math.max(0,Math.min(im.naturalWidth-c.width,Math.round(point[0]*im.naturalWidth-c.width/2))),y=Math.max(0,Math.min(im.naturalHeight-c.height,Math.round(point[1]*im.naturalHeight-c.height/2)));const ctx=c.getContext('2d');ctx.imageSmoothingEnabled=false;ctx.clearRect(0,0,c.width,c.height);ctx.drawImage(im,x,y,c.width,c.height,0,0,c.width,c.height);}}
async function render(){const r=current();if(!r)return;const g=++generation;const forced=$('candidate').value==='forced';const mode=forced||r.symbol_selected?395:135;const files=[0,135,mode];await Promise.all(['a','b','c'].map((id,i)=>new Promise(resolve=>{const im=$(id);im.onload=resolve;im.onerror=resolve;im.src=r.id+'_'+files[i]+'.png';if(im.complete)resolve();})));if(g!==generation)return;crop();$('position').textContent=' amostra '+r.frame+'/8';const q=r.checks[forced?1:2];$('metrics').textContent=`${r.size.join(' × ')} · B ${r.triangles.toLocaleString()} triângulos; símbolo ${r.forced_triangles.toLocaleString()}; elegível ${r.eligible} / ${r.source_triangles.toLocaleString()} fonte\nEscolha automática: ${r.symbol_selected?'símbolo':'anterior (fallback)'} · candidato exibido: ${forced?'forçado':'opcional'}\nRMSE ${q.foreground_rmse.toFixed(6)} · p99 ${q.p99_rgb.toFixed(6)} · divergência de cobertura ${q.coverage_mismatches} pixels · critério ${q.quality_gate?'PASSOU':'FALHOU'}`;}
for(const id of ['density','resolution','distance','scene','candidate'])$(id).onchange=()=>{if(id==='density'||id==='distance')scenes();render()};$('frame').oninput=render;
for(const id of ['a','b','c'])$(id).onclick=e=>{const rect=$(id).getBoundingClientRect();point=[(e.clientX-rect.left)/rect.width,(e.clientY-rect.top)/rect.height];crop()};
$('play').onclick=()=>{if(timer){clearInterval(timer);timer=null;$('play').textContent='Reproduzir capturas'}else{timer=setInterval(()=>{$('frame').value=(+$('frame').value+1)%9;render()},650);$('play').textContent='Pausar'}};window.onresize=crop;scenes();render();
</script></html>'''.replace('FIXED_ROWS',fixed_rows).replace('ROWS',rows).replace('DATA_PLACEHOLDER',data).replace('POLICY_RESULT', 'A calibração não encontrou benefício robusto: política conservou o anterior em todos os casos.' if not read('fit')['enabled_mask'] else 'A calibração habilitou classes de trabalho; a seleção registrada abaixo é congelada antes da validação.')
(R/'index.html').write_text(page)
print(json.dumps({k:v for k,v in result.items() if k not in ['rows','fixed_quality','fixed_timings','fixed_preparation','fixed_summary']},indent=2))
