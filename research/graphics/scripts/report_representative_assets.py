"""Build the local, self-contained comparison report from measured native GPU captures."""
import json, statistics
from pathlib import Path
ROOT=Path('artifacts/appearance-probe/assets')
NAMES={'rock':'Pedra detalhada','chest':'Baú multimaterial','tree':'Árvore'}
SCENES={'camera':'Câmera girando · longe','light':'Luz girando · longe','both':'Câmera + luz · longe','near':'Câmera + luz · perto','zoom':'Aproximar e afastar','seam':'Passagem por 360°'}

def median(xs): return statistics.median(xs) if xs else None

def main():
    data={name:json.loads((ROOT/name/'report.json').read_text()) for name in NAMES}
    summary={}
    verification=json.loads((ROOT/'verification.json').read_text())
    for name in data: data[name]['verification']=verification[name]
    for name,d in data.items():
        assert len(d['cases'])==114 and d['checks']['bounded_pairs']==114
        assert d['checks']['repeat_wrap_light_return']
        summary[name]={}
        for scene in SCENES:
            rows=[r for r in d['cases'] if r['scene']==scene]; timed=[r for r in rows if r['timings']]
            ratio=lambda slot,kind:median([r['timings'][slot][kind]['median']/r['timings'][0][kind]['median'] for r in timed if r['timings'][0][kind]['median']])
            summary[name][scene]={
                'gpu_original_over_full':ratio(1,'gpu'),'gpu_compact_over_full':ratio(2,'gpu'),
                'gpu_full_control_over_full':ratio(3,'gpu'),'serial_compact_over_full':ratio(2,'serial_frame'),
                'gpu_ms':[median([r['timings'][i]['gpu']['median'] for r in timed if r['timings'][i]['gpu']['median'] is not None]) for i in range(4)],
                'gpu_ratio_minmax':[min(r['timings'][2]['gpu']['median']/r['timings'][0]['gpu']['median'] for r in timed),max(r['timings'][2]['gpu']['median']/r['timings'][0]['gpu']['median'] for r in timed)] if timed[0]['timings'][0]['gpu']['median'] else None,
                'rmse16':median([r['compact_error']['rmse_surface'] for r in rows]),
                'rmse1':median([r['native_compact_error']['rmse_surface'] for r in rows]),
                'full_rmse1':median([r['native_full_error']['rmse_surface'] for r in rows]),
                'silhouette':median([r['silhouette_l1_relative'] for r in rows]),
                'temporal':median([r['temporal_residual'] for r in rows if r['temporal_residual'] is not None]),'timed_poses':len(timed)}
        d['summary']=summary[name]
        d['disk_bytes']={k:sum(p.stat().st_size for p in paths) for k,paths in {
            'mesh':[ROOT/name/'vertices.bin',ROOT/name/'indices.bin',ROOT/name/'asset.json'],
            'original':[ROOT/name/'atlas24.bin',ROOT/name/'atlas24.json'],
            'compact':[ROOT/name/'packed/atlas.bin',ROOT/name/'packed/atlas.json']}.items()}
    (ROOT/'summary.json').write_text(json.dumps(summary,indent=2))
    cards=''
    for name,d in data.items():
        cards+=f'<tr><th>{NAMES[name]}</th><td>{d["asset"]["triangles"]:,}</td><td>{d["geometry_bytes"]/2**20:.2f} MiB</td><td>75.00 MiB</td><td>{d["compact_bytes"]/2**20:.2f} MiB</td><td>{d["compact_bytes"]/d["geometry_bytes"]:.1f}×</td></tr>'
    template='''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Ignis · Assets representativos</title>
<style>
:root{color-scheme:dark;font:15px/1.5 system-ui;background:#10151d;color:#dce5ed}body{max-width:1440px;margin:0 auto;padding:30px}h1{font-size:34px;margin:5px 0}h2{font-size:20px}p{max-width:1100px;color:#aebdca}.eyebrow{color:#f0ae60;font-weight:700;letter-spacing:.13em;font-size:12px}table{border-collapse:collapse;width:100%;font-variant-numeric:tabular-nums}td,th{text-align:left;padding:10px;border-bottom:1px solid #303a49}th{color:#cdd8e3}.panel{background:#19222e;border:1px solid #303e4f;border-radius:12px;padding:18px;margin:20px 0}.controls{display:flex;gap:16px;flex-wrap:wrap;align-items:end}label{display:grid;gap:5px;color:#aebdca}select,button{font:inherit;color:#e2e8ef;background:#253348;border:1px solid #47617a;padding:8px;border-radius:6px}button{cursor:pointer}input[type=range]{width:min(540px,70vw);accent-color:#f0ae60}.images{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px}.images figure{margin:0;min-width:0}.images canvas{width:100%;height:auto;background:#222;image-rendering:pixelated}.images figcaption{font-weight:650;margin-bottom:7px}.stat{font-variant-numeric:tabular-nums;color:#f4c58e}.note{font-size:13px}.details{display:grid;grid-template-columns:1fr 1fr;gap:24px}a{color:#88bfee}#status{color:#ffad7b}@media(max-width:900px){.images{grid-template-columns:repeat(2,minmax(0,1fr))}.details{grid-template-columns:1fr}body{padding:15px}.table{overflow:auto}}
</style>
<body><div class="eyebrow">IGNIS / PESQUISA GRÁFICA / EAF-72</div><h1>O resumo compensa neste objeto?</h1>
<p>Três modelos autorados no Blender. Compare a geometria indexada com o atlas angular e sua versão compactada. A referência integra 64 amostras por pixel; o cronômetro mede 1 amostra por pixel, sem captura.</p>
<div class="panel table"><table><thead><tr><th>Objeto</th><th>Triângulos</th><th>Malha indexada</th><th>Atlas original</th><th>Compactado</th><th>Compacto / malha</th></tr></thead><tbody>__CARDS__</tbody></table><p class="note">Payload residente por representação; inclui índices e descritores. Não inclui render targets, overhead do driver nem RAM temporária. No laboratório as representações coexistem para comparação. Dados não são multiplicados por instância compatível.</p></div>
<div class="panel"><div class="controls">
<label>Objeto<select id="asset"><option value="rock">Pedra detalhada</option><option value="chest">Baú multimaterial</option><option value="tree">Árvore</option></select></label>
<label>Percurso<select id="scene"><option value="both">Câmera + luz · longe</option><option value="camera">Câmera girando · longe</option><option value="light">Luz girando · longe</option><option value="near">Câmera + luz · perto</option><option value="zoom">Aproximar e afastar</option><option value="seam">Passagem por 360°</option></select></label>
<label>Imagem<select id="sampling"><option value="native">1 amostra · corresponde ao custo medido</option><option value="integrated">16 amostras · inspeção com menos aliasing</option></select></label>
<label>Ampliação comum<select id="zoom"><option value="1">Imagem inteira</option><option value="2">Centro 2×</option></select></label>
</div><p><button id="back" aria-label="Quadro anterior">←</button> <button id="play">Reproduzir</button> <button id="next" aria-label="Próximo quadro">→</button> <span id="position"></span></p><input id="frame" type="range" min="0" max="20" value="0" aria-label="Quadro"><div id="pose" class="stat"></div><div id="status" role="status"></div></div>
<div class="images" id="images"></div>
<div class="panel details"><div><h2>Qualidade nesta pose</h2><div id="quality"></div><p class="note">RMSE em RGB linear na união dos pixels cobertos, não uma porcentagem de qualidade percebida. Cobertura mede a diferença absoluta de alpha relativa à área da referência. A referência também tem erro residual de integração.</p></div><div><h2>Custo neste percurso</h2><div id="timing"></div><p class="note">Mediana das razões por pose, 9 blocos intercalados de 32 desenhos por método, com limpeza de cor/profundidade a cada desenho. Controle repete a geometria completa para expor variação. Tempo serial inclui submissão e espera; não é FPS de gameplay. Reproduzir aqui mostra capturas pré-calculadas.</p></div></div>
<details class="panel"><summary>Erros e limites do ensaio</summary><div class="images" id="errors"></div><p>Atlas de 24 vistas × 24 luzes, 128² texels. Elevações de câmera 55° e de luz 35° fixas. GGX direto com sete perfis de rugosidade/metallic, cores por vértice. A árvore usa folhas geométricas opacas estáticas. Sem texturas PBR, vento, transmissão, sombras projetadas ou GI. A direção de observação usada no material é comum ao objeto, inclusive na referência. Altura única pode apagar sobreposições e falhar na desoclusão. Este conjunto não representa uma média AAA.</p><div id="memory"></div><p><a href="assets.blend">Fonte Blender editável</a> · <a href="review-side.png">Revisão dos modelos</a> · <a href="summary.json">Resumo numérico</a> · <a href="verification.json">Diferenças de compactação e verificação de linhas</a></p></details>
<script>
const data=__DATA__, names=__NAMES__;
const $=id=>document.getElementById(id);let timer=null,token=0;
function rows(){return data[$('asset').value].cases.filter(r=>r.scene===$('scene').value)}
function fmt(x,n=4){return x===null||x===undefined?'indisponível':Number(x).toFixed(n)}
function picture(container,path,label,drawToken){const f=document.createElement('figure'),c=document.createElement('canvas'),caption=document.createElement('figcaption');caption.textContent=label;c.width=256;c.height=256;f.append(caption,c);container.append(f);const image=new Image();image.onload=()=>{if(drawToken!==token)return;const z=Number($('zoom').value),w=256/z,x=(256-w)/2;const ctx=c.getContext('2d');ctx.imageSmoothingEnabled=false;ctx.drawImage(image,x,x,w,w,0,0,256,256)};image.onerror=()=>{if(drawToken===token)$('status').textContent='Falha ao carregar '+path};image.src=path}
function render(){const r=rows(),i=Math.max(0,Math.min(Number($('frame').value),r.length-1));$('frame').value=i;$('frame').max=r.length-1;const row=r[i],name=$('asset').value,d=data[name],s=d.summary[$('scene').value],native=$('sampling').value==='native',prefix=native?'native_':'';const t=++token;$('status').textContent='';$('images').replaceChildren();$('errors').replaceChildren();$('position').textContent=`${i+1}/${r.length}`;$('pose').textContent=`${names[name]} · câmera ${fmt(row.pose.camera,1)}° · luz ${fmt(row.pose.light,1)}° · distância ${fmt(row.pose.distance,2)}`;
for(const [method,label] of [['reference','Referência integrada · 64 spp'],[prefix+'full','Geometria completa'],[prefix+'original','Atlas original · 75 MiB'],[prefix+'compact','Atlas compactado']])picture($('images'),`${name}/${row.id}_${method}.png`,label,t);
for(const [method,label] of [['error','Compactado × referência · erro ampliado 8× / 16 spp'],['packing','Compactado × original · erro ampliado 8× / 16 spp']])picture($('errors'),`${name}/${row.id}_${method}.png`,label,t);
const e=row[prefix+'compact_error'],f=row[prefix+'full_error'],a=row[prefix+'original_error'];$('quality').textContent=`RMSE: geometria ${fmt(f.rmse_surface)} · original ${fmt(a.rmse_surface)} · compacto ${fmt(e.rmse_surface)}. Erro de cobertura: ${fmt(100*row[native?'native_silhouette_l1_relative':'silhouette_l1_relative'],1)}%. Erro introduzido pela compactação (16 spp): ${fmt(row.packing_error.rmse_surface,8)}. Na varredura de ${native?'1':'16'} spp, maior diferença de canal entre os PNGs original/compacto: ${d.verification.pixel_checks[native?'native':'integrated'].max_png_delta}/255; maior sequência vertical acima de 1 nível: ${d.verification.pixel_checks[native?'native':'integrated'].longest_vertical_run_over_one_level} pixel.`;
$('timing').textContent=`GPU: geometria ${fmt(s.gpu_ms[0])} ms · original ${fmt(s.gpu_ms[1])} ms · compacto ${fmt(s.gpu_ms[2])} ms. Razão compacto/geometria ${fmt(s.gpu_compact_over_full,3)}× (menor é melhor); original/geometria ${fmt(s.gpu_original_over_full,3)}×. Controle geometria/geometria ${fmt(s.gpu_full_control_over_full,3)}×. Tempo serial compacto/geometria ${fmt(s.serial_compact_over_full,3)}×. ${s.timed_poses} poses medidas.`;
$('memory').textContent=`Disco: malha ${fmt(d.disk_bytes.mesh/1048576,2)} MiB; atlas original ${fmt(d.disk_bytes.original/1048576,2)} MiB; compacto com metadados ${fmt(d.disk_bytes.compact/1048576,2)} MiB. Bake ${fmt(d.bake_seconds,2)} s; compactação ${fmt(d.pack_seconds,2)} s. Carregamento/upload observado: original ${fmt(d.original_load_ms,1)} ms; compacto ${fmt(d.compact_load_ms,1)} ms.`;
}
function step(delta){const n=rows().length;$('frame').value=(Number($('frame').value)+delta+n)%n;render()}
$('asset').onchange=$('scene').onchange=()=>{$('frame').value=0;render()};$('sampling').onchange=$('zoom').onchange=$('frame').oninput=render;
$('back').onclick=()=>step(-1);$('next').onclick=()=>step(1);$('play').onclick=()=>{if(timer){clearInterval(timer);timer=null;$('play').textContent='Reproduzir'}else{timer=setInterval(()=>step(1),150);$('play').textContent='Pausar'}};render();
</script></body></html>'''
    output=template.replace('__CARDS__',cards).replace('__DATA__',json.dumps(data,separators=(',',':'))).replace('__NAMES__',json.dumps(NAMES))
    (ROOT/'index.html').write_text(output)
    print(json.dumps(summary,indent=2))

if __name__=='__main__': main()
