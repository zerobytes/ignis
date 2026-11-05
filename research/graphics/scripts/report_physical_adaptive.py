"""Build the offline physical-pixel zoom comparison from actual GPU captures."""
import json
from pathlib import Path
ROOT = Path('artifacts/appearance-probe/adaptive-assets')

def main():
    data = {name: json.loads((ROOT/name/'report.json').read_text()) for name in ['rock','chest','tree']}
    html = '''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Ignis · seleção por pixel físico</title>
<style>
*{box-sizing:border-box}body{background:#0d141d;color:#e7ecf3;font:15px/1.55 system-ui,sans-serif;margin:28px auto;padding:0 24px;max-width:1500px}h1{font-size:32px;line-height:1.2}h2{font-size:18px}p{max-width:1100px}.eyebrow{color:#f1bd78;font-size:12px;letter-spacing:2px}.panel{background:#151f2c;border:1px solid #334358;border-radius:10px;padding:18px;margin:18px 0}.controls{display:flex;gap:18px;flex-wrap:wrap;align-items:end}label{display:grid;gap:5px;color:#b8c7d6}button,select{padding:8px;background:#26374a;color:#f3f6fb;border:1px solid #5a7088;border-radius:6px;font:inherit}button{cursor:pointer}input{width:min(700px,75vw);accent-color:#f1bd78}.images{display:flex;gap:18px;flex-wrap:wrap;align-items:start}.image-scroll{overflow:auto;max-width:100%}figure{margin:0;flex:none}figcaption{font-weight:650;margin-bottom:8px}.images img{display:block;max-width:none;image-rendering:pixelated;background:#222}.stat{color:#f1bd78;font-variant-numeric:tabular-nums}#decision{font-size:22px;font-weight:650}#status{color:#ffac8f}.note{font-size:13px;color:#b2c2d3}table{border-collapse:collapse}td,th{padding:7px 18px 7px 0;text-align:left}a{color:#98c8ff}code{color:#d0dfed}
</style>
<body><div class="eyebrow">IGNIS · EAF-73 · EXPERIMENTO LOCAL</div>
<h1>Detalhe visível fica como geometria</h1>
<p>A câmera afasta e retorna pelo mesmo caminho. O seletor considera a resolução física do render e só permite resumo quando <b>todos os triângulos têm tamanho projetado limitado a menos de 1 pixel</b> e o suporte planar do filtro do atlas também cabe nesse limite. A entrada usa 0,75 px; o retorno à geometria ocorre em 1 px.</p>
<div class="panel controls">
<label>Objeto<select id="asset"><option value="rock">Pedra detalhada</option><option value="chest">Baú multimaterial</option><option value="tree">Árvore</option></select></label>
<label>Resolução do render<select id="resolution"><option value="0">3456 × 2234 · física da tela principal</option><option value="1">1728 × 1117 · render reduzido explícito</option></select></label>
<label>Exibição<select id="zoom"><option value="1">1 pixel da imagem = 1 pixel físico</option><option value="2">Ampliação de inspeção: 2×</option><option value="4">Ampliação de inspeção: 4×</option></select></label>
<button id="play">Reproduzir ida e volta</button><button id="transition">Ir à primeira troca</button>
<div><input id="frame" type="range" min="0" value="0" step="1"><p id="pose" class="stat"></p></div></div>
<div class="panel"><div id="decision"></div><p id="bounds"></p><p id="density"></p><p id="display" class="note"></p><p id="status"></p></div>
<div class="image-scroll"><div id="images" class="images"></div></div>
<div class="panel"><h2>Qualidade e custo nesta posição</h2><p id="quality"></p><p id="timing"></p><p id="memory"></p></div>
<div class="panel"><h2>Como ler este teste</h2>
<p>As imagens são <b>recortes centrais de 512 × 512 pixels físicos</b>, com a projeção da resolução escolhida. Não são um render de 512² ampliado para simular uma tela maior. O objeto inteiro cabe no recorte durante este percurso. O tamanho CSS é ajustado ao <code>devicePixelRatio</code> do navegador; a ampliação de inspeção é sempre explícita.</p>
<p>A referência integra 16 amostras por pixel. Geometria, adaptativo e resumo forçado usam 1 amostra por pixel. Assim, diferenças também incluem aliasing do render de uma amostra. O teste registra convergência da referência contra 64 amostras em posições selecionadas. O resumo forçado é controle: ele pode perder detalhe próximo e não é a escolha adaptativa.</p>
<p>O bound usa a maior aresta e a caixa do objeto, incluindo partes ocultas: é conservador. Ainda não há divisão do objeto em regiões independentes. Isso posterga especialmente a troca do baú, que contém faces grandes. O suporte do filtro é medido no plano do bake; não limita descontinuidades de profundidade. Subpixel é requisito geométrico, não garantia de erro visual zero.</p>
<p>O diagnóstico usa 4 × 4 amostras com profundidade por pixel para contar IDs de triângulos visíveis. É uma amostragem limitada a 16 IDs/pixel, não uma contagem exata. Ele percorre a malha somente para avaliar o experimento; o seletor usa oito cantos e bounds pré-calculados, sem essa varredura.</p>
<p>Tempos GPU medem apenas o passe do recorte512²: 9 blocos de32 desenhos, ordem alternada, sem captura/diagnóstico/PNG. Controle duplica o desenho da geometria para expor ruído. Não representam FPS ou custo de um jogo completo. As duas representações permanecem residentes. Câmera e luz mantêm ângulos fixos nesta rodada para isolar a aproximação. As distâncias usam unidades do experimento, com modelos normalizados ao raio1,22; não são distâncias medidas em metros de assets reais.</p>
<p>O slider reproduz capturas e decisões da trajetória registrada, incluindo a histerese da ida e da volta. Redimensionar esta página muda a apresentação, não recalcula as capturas. As duas resoluções usam exatamente as mesmas distâncias.</p>
</div>
<script>
const DATA=__DATA__;
const names={rock:'Pedra detalhada',chest:'Baú multimaterial',tree:'Árvore'};
const $=id=>document.getElementById(id);
let interval=null,epoch=0;
const fmt=(n,d=3)=>Number(n).toFixed(d);
function rows(){return DATA[$('asset').value].cases.filter(r=>r.resolution===Number($('resolution').value))}
function show(){
 const data=DATA[$('asset').value],list=rows();$('frame').max=list.length-1;
 const r=list[Number($('frame').value)],dpr=window.devicePixelRatio||1,zoom=Number($('zoom').value),e=++epoch;
 $('pose').textContent=`${names[$('asset').value]} · ${r.way==='out'?'Afastando':'Aproximando'} · distância ${fmt(r.pose.distance,2)} unidades · posição ${Number($('frame').value)+1}/${list.length}`;
 $('decision').textContent=r.summary?'Resumo · 2 triângulos desenhados':`Geometria · ${data.triangles.toLocaleString('pt-BR')} triângulos desenhados`;
 const q=Math.max(r.triangle_bound_px,r.filter_bound_px);
 const reason=q>=1?'Detalhe ou filtro ainda alcança 1 pixel.':q<0.75?'Ambos os limites ficaram abaixo de 0,75 pixel.':'Faixa de histerese: preserva a decisão do percurso.';
 $('bounds').textContent=`Limite de aresta: ${fmt(r.triangle_bound_px)} px · suporte planar do filtro: ${fmt(r.filter_bound_px)} px. ${reason}${r.switched?' Troca de representação nesta posição.':''}`;
 $('density').textContent=`Visibilidade amostrada: ${fmt(r.density.mean_ids_per_covered_pixel,2)} triângulos/pixel ocupado · ${fmt(r.density.fraction_multiple*100,1)}% dos pixels ocupados contêm mais de um ID · ${r.density.covered_pixels} pixels ocupados. Maior aresta visível amostrada: ${fmt(r.density.max_visible_edge_px)} px.`;
 $('display').textContent=`Render ${r.viewport.join(' × ')} · recorte ${data.window}² pixels · DPR do navegador ${dpr} · imagem ${fmt(data.window*zoom/dpr,1)} pixels CSS de lado · ${zoom}× físico${zoom===1?' (sem ampliação)':' (ampliação de inspeção)'}.`;
 $('quality').textContent=`RMSE RGB linear contra referência integrada: geometria ${fmt(r.full_error.rmse_surface,5)} · adaptativo ${fmt(r.adaptive_error.rmse_surface,5)} · resumo forçado ${fmt(r.forced_error.rmse_surface,5)}. Erro de cobertura (alpha): adaptativo ${fmt(r.adaptive_error.alpha_rmse,5)}. Menor é melhor; não é uma nota perceptual.`;
 const t=r.timings;
 $('timing').textContent=t&&t[0].gpu.median!==null?`GPU/passe: geometria ${fmt(t[0].gpu.median,4)} ms · resumo forçado ${fmt(t[1].gpu.median,4)} ms · adaptativo ${fmt(t[2].gpu.median,4)} ms · controle geometria ${fmt(t[3].gpu.median,4)} ms. Seleção geométrica e diagnóstico não estão incluídos neste cronômetro.`:'GPU: não cronometrada nesta posição; medidas nos extremos e nas trocas. Diagnóstico e captura ficam fora do timing.';
 $('memory').textContent=`Payload residente: malha ${fmt(data.geometry_bytes/1048576,2)} MiB + atlas compactado ${fmt(data.atlas_bytes/1048576,2)} MiB. Seleção não reduz a memória residente neste experimento.`;
 $('status').textContent='Carregando capturas…';
 const asset=$('asset').value;
 Promise.all([['reference','Referência integrada · 16 spp'],['full','Geometria · 1 spp'],['adaptive','Seleção adaptativa · 1 spp'],['forced','Resumo forçado · 1 spp']].map(([suffix,label])=>new Promise((resolve,reject)=>{
  const figure=document.createElement('figure'),caption=document.createElement('figcaption'),img=new Image();caption.textContent=label;img.alt=label;
  img.style.width=`${data.window*zoom/dpr}px`;img.style.height=`${data.window*zoom/dpr}px`;
  img.onload=()=>{figure.append(caption,img);resolve(figure)};img.onerror=reject;img.src=`${asset}/${r.id}_${suffix}.png`;
 }))).then(figures=>{if(e===epoch){$('images').replaceChildren(...figures);$('status').textContent=''}}).catch(()=>{if(e===epoch)$('status').textContent='Falha ao carregar uma captura.'});
}
function stop(){if(interval!==null)clearInterval(interval);interval=null;$('play').textContent='Reproduzir ida e volta'}
$('asset').onchange=()=>{stop();$('frame').value='0';show()};
$('resolution').onchange=show;$('zoom').onchange=show;$('frame').oninput=show;
$('transition').onclick=()=>{stop();$('frame').value=String(rows().findIndex(r=>r.summary));show()};
$('play').onclick=()=>{if(interval!==null){stop();return} $('play').textContent='Pausar';interval=setInterval(()=>{$('frame').value=String((Number($('frame').value)+1)%rows().length);show()},450)};
window.addEventListener('resize',show);
show();
</script></body></html>'''
    (ROOT/'index.html').write_text(html.replace('__DATA__',json.dumps(data,separators=(',',':'))))
    print(ROOT/'index.html')

if __name__=='__main__':main()
