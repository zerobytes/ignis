#!/usr/bin/env python3
"""Same meshes, global versus offline hierarchical certificate selection."""
import json
from pathlib import Path
from statistics import median

OUT = Path(__file__).resolve().parents[1] / 'artifacts/appearance-probe/certificate-hierarchy'

def main():
    data = json.loads((OUT / 'results.json').read_text())
    for row in data['quality'] + data['timings']:
        row['distance'] = 2.7 if abs(row['distance'] - 2.7) < 1e-5 else row['distance']
    builds = json.loads((OUT / 'certificates.json').read_text())
    assert data['hierarchical_certificates'] and data['adaptive_fallback']
    assert len(data['quality']) == 144 and len(data['timings']) == 1024
    cohorts = []
    for factor in [1, 2, 10, 100]:
        for width in [1728, 3456]:
            for distance in [2.7, 64.]:
                samples = [x for x in data['timings'] if (x['factor'], x['width'], x['distance']) == (factor, width, distance)]
                c = dict(factor=factor, width=width, distance=distance)
                for mode in ['source', 'old', 'stable', 'base']:
                    a = [x for x in samples if x['mode'] == mode]
                    assert len(a) == 16
                    c[mode] = {k: median(x['timing'][k] for x in a) for k in ['gpu_ms', 'cpu_encode_ms', 'total_batch_ms_per_frame', 'proxy_frames']}
                    pairs = []
                    for r in range(8):
                        p = [x['timing']['total_batch_ms_per_frame'] for x in a if x['round'] == r]
                        pairs.append(abs(p[0] - p[1]) / min(p) * 100)
                    c[mode]['duplicate_percent'] = median(pairs)
                c['favorable_rounds'] = sum(median(x['timing']['total_batch_ms_per_frame'] for x in samples if x['round'] == r and x['mode'] == 'stable') < median(x['timing']['total_batch_ms_per_frame'] for x in samples if x['round'] == r and x['mode'] == 'old') for r in range(8))
                cohorts.append(c)
    failures = {k: [dict(factor=x['factor'], width=x['width'], distance=x['distance'], frame=x['frame'], metrics=x[k]) for x in data['quality'] if not x[k]['pass']] for k in ['density', 'old_base', 'stable_base', 'old_source', 'stable_source']}
    build_summary = [{k: v for k, v in b.items() if k != 'audit'} for b in builds]
    summary = dict(cohorts=cohorts, failures=failures, builds=build_summary, adapters=data['adapters'])
    (OUT / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    html = '''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Certificados locais compilados</title><style>
body{font:16px system-ui;background:#111722;color:#edf3fa;margin:24px}h1{font-size:28px}p{max-width:1150px;line-height:1.5}a{color:#7dddbb}label{display:inline-block;margin:8px}select{font:inherit;background:#26374e;color:white;padding:6px}section{display:flex;gap:16px}figure{flex:1;min-width:0;margin:0}figcaption{margin:10px 0}.view{height:460px;overflow:auto;background:black}img{display:block;max-width:none}table{border-collapse:collapse;font-size:14px}td,th{border:1px solid #4a5e77;padding:8px;text-align:right}#metric{white-space:pre-wrap;font:14px monospace}.note{color:#adc0d7}@media(max-width:800px){section{display:block}.view{height:300px}}</style><body>
<h1>Mesma geometria, uma decisão mais precisa</h1>
<p>Wooden Stool02: banco de11.146 triângulos. A fonte1×/2×/10×/100× subdivide a mesma superfície, sem acrescentar relevo. O compilador recupera uma malha de cerca de11mil triângulos. Este teste muda apenas a decisão de quando é seguro desenhá-la: certificado global anterior versus hierarquia preparada em build time.</p>
<p>Se o limite de um grupo passa, seus descendentes não são consultados. Caso contrário, examinamos grupos menores. Mantemos erro≤0,15pixel físico e normais≤0,000101; folha reprovada ou128 consultas esgotadas retorna à fonte completa. Mesmos shaders/materiais nos quatro caminhos. UV/texturas/PBR originais não incluídos.</p>
<label>Subdivisão <select id="factor"><option>1</option><option>2</option><option>10</option><option selected>100</option></select></label>
<label>Pixels <select id="resolution"><option value="1728">1728 ×1117</option><option value="3456" selected>3456 ×2234</option></select></label>
<label>Câmera <select id="distance"><option value="0">Perto (2,7)</option><option value="1">Longe (64)</option></select></label>
<label>Órbita / luz <input id="frame" type="range" min="0" max="7" value="1"></label>
<label>Referência <select id="reference"><option value="base">Malha importada1× (11.146T)</option><option value="source">Fonte subdividida completa</option></select></label>
<label>Seleção <select id="mode"><option value="stable">Hierarquia local</option><option value="old">Certificado global anterior</option></select></label>
<label><input id="fit" type="checkbox" checked>Visão geral</label><p id="outcome"></p><p id="scale" class="note"></p>
<section><figure><figcaption id="referenceTitle"></figcaption><div class="view"><img id="source" alt="Referência"></div></figure><figure><figcaption id="candidateTitle"></figcaption><div class="view"><img id="candidate" alt="Compilado adaptativo"></div></figure></section><p id="metric"></p>
<h2>Comparação completa</h2><p>Milissegundos por frame de lote, mediana de16 amostras/caminho,128frames/amostra, oito rodadas alternadas com duplicatas e32poses. Total inclui seleção, codificação, submissão e espera; não representa FPS de gameplay. GPU é intervalo de timestamps do lote. As imagens são capturas offscreen, não renderização interativa.</p>
<table><thead><tr><th>Fator</th><th>Total fonte densa</th><th>Total global anterior</th><th>Total hierarquia</th><th>Total importada1×</th><th>GPU fonte</th><th>GPU global</th><th>GPU hierarquia</th><th>CPU hierarquia</th><th>Proxy global/hierarquia de128</th><th>Duplicatas hierarquia</th><th>Rodadas favoráveis/8</th></tr></thead><tbody id="timings"></tbody></table>
<h2>Qualidade</h2><p>Gates:RMSE≤0,005,p99≤0,02,cobertura≤0,0005. Passar não significa pixels idênticos. As falhas da fonte subdividida contra a própria superfície importada também ficam expostas. Falhas1spp conhecidas continuam abertas; diagnóstico integrado no ensaio anterior não as apaga.</p><div id="qualityTable"></div>
<h2>Dados compilados</h2><div id="builds"></div><p>A hierarquia reside naCPU; fonte e proxy continuam naGPU. Esses bytes adicionais não substituem a fonte de fallback e não representam a memória total. Não foi medido RSS. <a href="certificates.json">Build e auditoria</a> · <a href="results.json">Medições</a> · <a href="../external-recovery/index.html">Experimento anterior e referência integrada</a> · <a href="https://polyhaven.com/a/wooden_stool_02">Wooden Stool02, Kuutti Siitonen, CC0</a></p>
<script>
const data=__DATA__, $=id=>document.getElementById(id), ms=x=>x.toFixed(5);
function update(){
 const f=+$('factor').value,w=+$('resolution').value,di=+$('distance').value,d=di===0?2.7:64,frame=+$('frame').value,mode=$('mode').value,ref=$('reference').value,fit=$('fit').checked;
 const row=data.rows.find(x=>x.factor===f&&x.width===w&&x.distance===d&&x.frame===frame);
 const q=row[(mode==='stable'?'stable':'old')+'_'+(ref==='base'?'base':'source')];
 $('source').src=`${f}-${w}-${di}-${frame}-${ref}.png`; $('candidate').src=`${f}-${w}-${di}-${frame}-${mode}.png`;
 for(const id of ['source','candidate']){const im=$(id);im.style.width=fit?'100%':`${w/devicePixelRatio}px`;im.onload=()=>{const v=im.parentElement;v.scrollLeft=fit?0:Math.max(0,(im.clientWidth-v.clientWidth)/2);v.scrollTop=fit?0:Math.max(0,(im.clientHeight-v.clientHeight)/2);};}
 $('referenceTitle').textContent=ref==='base'?'Malha importada1× —11.146 triângulos':`Fonte completa —${row.source_triangles} triângulos`;
 $('candidateTitle').textContent=mode==='stable'?'Mesma malha compilada + hierarquia local':'Mesma malha compilada + certificado global';
 $('scale').textContent=fit?'Visão geral reduzida; desmarque para inspecionar pixels físicos.':`1 pixel renderizado por pixel físico, DPR${devicePixelRatio}. Role para inspecionar a imagem.`;
 $('outcome').textContent=q.pass?'Passa nos limites numéricos desta captura.':'FALHA nos limites desta captura; mantida no relatório.';
 const fallback=mode==='stable'?row.fallback:row.old_fallback;
 $('metric').textContent=`Entrada ${row.source_triangles}T; proxy ${row.triangles}T; desenhados ${fallback?row.source_triangles:row.triangles}T.\nFallback: ${fallback?'sim':'não'}; consultas locais: ${row.hierarchy.visits}; limite aceito hierarquia: ${row.hierarchy.accepted_error_px.toFixed(6)}px.\nRMSE ${q.rmse.toExponential(5)}, p99 ${q.p99.toExponential(5)}, cobertura ${q.coverage_error.toExponential(5)}.`;
 $('timings').innerHTML=data.summary.cohorts.filter(c=>c.width===w&&c.distance===d).map(c=>`<tr><td>${c.factor}×</td>${['source','old','stable','base'].map(m=>`<td>${ms(c[m].total_batch_ms_per_frame)}</td>`).join('')}${['source','old','stable'].map(m=>`<td>${ms(c[m].gpu_ms)}</td>`).join('')}<td>${ms(c.stable.cpu_encode_ms)}</td><td>${c.old.proxy_frames}/${c.stable.proxy_frames}</td><td>${c.stable.duplicate_percent.toFixed(1)}%</td><td>${c.favorable_rounds}</td></tr>`).join('');
 $('qualityTable').innerHTML=Object.entries(data.summary.failures).map(([name,rows])=>`<p>${({density:'Fonte subdividida × importada1×',old_base:'Global anterior × importada1×',stable_base:'Hierarquia × importada1×',old_source:'Global anterior × fonte subdividida',stable_source:'Hierarquia × fonte subdividida'})[name]}: ${rows.length} falhas de144. ${rows.map(x=>`${x.factor}×/${x.width}/d${x.distance}/pose${x.frame}`).join('; ')}</p>`).join('');
}
for(const id of ['factor','resolution','distance','frame','mode','reference','fit'])$(id).addEventListener('input',update);
window.addEventListener('resize',update);
$('builds').innerHTML='<table><tr><th>Fator</th><th>Regiões</th><th>Nós</th><th>Arquivo KiB</th><th>ResidenteCPU KiB</th><th>Build+autovalidação ms</th><th>Load+validação ms</th></tr>'+data.summary.builds.map(b=>`<tr><td>${b.factor}×</td><td>${b.regions}</td><td>${b.nodes}</td><td>${(b.file_bytes/1024).toFixed(2)}</td><td>${(b.resident_bytes/1024).toFixed(2)}</td><td>${b.build_ms.toFixed(3)}</td><td>${b.load_ms.toFixed(3)}</td></tr>`).join('')+'</table>';
update();
</script></body></html>'''
    (OUT / 'index.html').write_text(html.replace('__DATA__', json.dumps(dict(summary=summary, rows=data['quality']))))
    print(json.dumps(summary, indent=2))

if __name__ == '__main__':
    main()
