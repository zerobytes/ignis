"""Publish the E02 visual review without mutating the model or its baseline."""
import hashlib
import html
import json
from pathlib import Path

PROJECT=Path(__file__).resolve().parents[1]
ROOT=PROJECT/'artifacts/house-e02-rebuild'

def main():
    data=json.loads((PROJECT/'docs/references/image-to-3d/e02-differences.json').read_text())
    actual=hashlib.sha256((ROOT/'detail.png').read_bytes()).hexdigest()
    assert actual==data['native_render_sha256']
    buttons=''.join('<button data-id="'+f['id']+'">'+html.escape(f['id']+' · '+f['title'])+'</button>' for f in data['findings'])
    payload=json.dumps(data,ensure_ascii=False).replace('<','\\u003c')
    page='''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>E02 — jogo das diferenças</title><style>body{margin:0;background:#242b28;color:#eee9df;font:15px system-ui}header{padding:20px;background:#303d36}h1{margin:0;font-size:24px}a{color:#c0e0cd}p{max-width:1100px;line-height:1.5}main{display:grid;grid-template-columns:300px 1fr}nav{padding:12px;max-height:82vh;overflow:auto}button{display:block;text-align:left;width:100%;background:#34433b;color:#ddd;border:1px solid #6d8373;border-radius:4px;padding:10px;margin-bottom:6px;cursor:pointer}button[aria-current=true]{background:#86693f;color:white}article{padding:20px;min-width:0}.pair{display:grid;grid-template-columns:1fr 1fr;gap:15px}.pair figure{margin:0;background:#aaa696;min-height:200px;display:flex;flex-direction:column;justify-content:center}.pair img{width:100%;max-height:480px;object-fit:contain}figcaption{padding:8px;background:#324039}h2{font-size:22px}h3{font-size:15px;color:#c9dcca}#classification{color:#edcb94}#ids{color:#aabfb0}footer{padding:20px;border-top:1px solid #68776c}@media(max-width:850px){main{grid-template-columns:1fr}nav{max-height:220px}.pair{grid-template-columns:1fr 1fr}article{padding:12px}}</style><header><h1>Jogo das diferenças · E02</h1><p>18 diferenças com evidência e causa distinguida: análise ambígua/insuficiente, execução e coerência espacial. A casa não foi alterada nesta revisão.</p><a href="orbit.html">Girar o modelo em 3D</a> · <a href="index.html">Todos os recortes e estados</a> · <a href="review-back.png">Verso nativo</a> · <a href="review-top.png">Superior nativa</a></header><main><nav>'''+buttons+'''</nav><article><div id="classification"></div><h2 id="title"></h2><div class="pair"><figure><img id="ref"><figcaption id="refCaption"></figcaption></figure><figure><img id="render"><figcaption id="renderCaption"></figcaption></figure></div><h3>Na referência</h3><p id="reference"></p><h3>No modelo</h3><p id="candidate"></p><h3>Por que aconteceu</h3><p id="cause"></p><h3>O que corrigir na análise</h3><p id="analysis"></p><h3>Próxima ação construtiva</h3><p id="action"></p><p id="ids"></p></article></main><footer>Contagens e diagnósticos são revisão visual, sem certificado determinístico ou percentual de fidelidade. Recortes normais usam a mesma janela de pixels; DF01 usa uma vista auxiliar identificada, sem comparação de score. O visualizador orbital renderiza a geometria salva em WebGL com iluminação própria; as imagens desta página são capturas nativas da Ignis.</footer><script id="data" type="application/json">'''+payload+'''</script><script>
const review=JSON.parse(document.getElementById('data').textContent);
function select(id){const f=review.findings.find(x=>x.id===id);if(!f)return;for(const [key,val] of Object.entries({title:f.title,classification:f.id+' · '+f.classification,reference:f.reference,candidate:f.render,cause:f.cause,analysis:f.analysis_revision,action:f.next_action,ids:f.items.join(' · ')}))document.getElementById(key).textContent=val;
const spatial=f.id==='DF01';document.getElementById('ref').src=spatial?'reference.png':f.evidence+'-ref.png';document.getElementById('render').src=spatial?'review-left.png':f.evidence+'-render.png';document.getElementById('refCaption').textContent=spatial?'Referência completa — câmera original':f.evidence+' — referência';document.getElementById('renderCaption').textContent=spatial?'Captura nativa lateral — outra câmera; fundo de estúdio removido na cópia de inspeção':f.evidence+' — captura nativa, mesma janela';document.querySelectorAll('[data-id]').forEach(b=>b.setAttribute('aria-current',b.dataset.id===id));history.replaceState(null,'','#'+id);}
document.querySelectorAll('[data-id]').forEach(b=>b.onclick=()=>select(b.dataset.id));select(location.hash.slice(1)||'DF01');
</script></html>'''
    (ROOT/'differences.html').write_text(page)
    print(ROOT/'differences.html')

if __name__=='__main__':main()
