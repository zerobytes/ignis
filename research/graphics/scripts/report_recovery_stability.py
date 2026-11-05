#!/usr/bin/env python3
"""Compare recovery cost and fidelity against both dense input and canonical surface."""
import json
import sys
from pathlib import Path
from statistics import median

ROOT = Path(__file__).resolve().parents[1]
EXTERNAL = '--external' in sys.argv
BORDERS = '--borders' in sys.argv or EXTERNAL
OUT = ROOT / 'artifacts/appearance-probe' / ('external-recovery' if EXTERNAL else 'border-recovery' if BORDERS else 'recovery-stability')

def main():
    data = json.loads((OUT/'results.json').read_text())
    builds = {str(f): json.loads((OUT/f'build-{f}.json').read_text()) for f in [1,2,10,100]}
    quality, cohorts = [], []
    for f in [1,2,10,100]:
        rows = [x for x in data['quality'] if x['factor']==f]
        assert len(rows)==36
        q = {'factor':f, 'triangles':rows[0]['triangles'], 'old_triangles':rows[0]['old_triangles']}
        for m in ['density','old_source','stable_source','old_base','stable_base']:
            q[m] = {'failures':sum(not x[m]['pass'] for x in rows), **{k:max(x[m][k] for x in rows) for k in ['rmse','p99','max','coverage_error']}}
        quality.append(q)
        for w in [1728,3456]:
            for d in sorted({x['distance'] for x in data['quality']}):
                c={'factor':f,'width':w,'distance':d}
                for mode in ['source','old','stable','base']:
                    a=[x for x in data['timings'] if x['factor']==f and x['width']==w and x['distance']==d and x['mode']==mode]
                    assert len(a)==16
                    c[mode]={k:median(x['timing'][k] for x in a) for k in ['gpu_ms','cpu_encode_ms','total_batch_ms_per_frame','proxy_frames']}
                    pairs=[]
                    for r in range(8):
                        p=[x['timing']['total_batch_ms_per_frame'] for x in a if x['round']==r]
                        pairs.append(abs(p[0]-p[1])/min(p)*100)
                    c[mode]['duplicate_wall_percent']=median(pairs)
                cohorts.append(c)
    summary={'builds':builds,'quality':quality,'cohorts':cohorts,'adapters':data['adapters']}
    (OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    html='''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Recuperação estável da pedra</title><style>body{font:16px system-ui;background:#111722;color:#edf3fa;margin:24px}h1{font-size:28px}p{max-width:1100px;line-height:1.5}a{color:#7dddbb}label{display:inline-block;margin:8px}select{font:inherit;background:#26374e;color:white;padding:6px}section{display:flex;gap:16px}figure{flex:1;min-width:0;margin:0}figcaption{margin:10px 0}.view{height:460px;overflow:auto;background:black}img{display:block;max-width:none}table{border-collapse:collapse;font-size:14px}td,th{border:1px solid #4a5e77;padding:8px;text-align:right}#metric{white-space:pre-wrap;font:14px monospace}.note{color:#adc0d7}@media(max-width:800px){section{display:block}.view{height:300px}}</style><body>
<h1>Compilar a mesma superfície sem carregar subdivisão desnecessária</h1><p>O compilador recalcula o plano e o campo de normais usando pontos mais afastados da própria região. Todas as faces são revalidadas sob os mesmos limites. A escolha entre anterior e refit ocorre no build; o frame desenha o pacote pronto. Sem atlas nem busca por pixel neste ensaio.</p>
<label>Subdivisão <select id="factor"><option>1</option><option>2</option><option>10</option><option selected>100</option></select></label><label>Pixels <select id="resolution"><option value="1728">1728 × 1117</option><option value="3456" selected>3456 × 2234</option></select></label><label>Câmera <select id="distance"><option value="0">Perto (2,7)</option><option value="1">Longe (64)</option></select></label><label>Órbita / luz <input id="frame" type="range" min="0" max="3" value="0"></label><label>Referência <select id="reference"><option value="base">Pedra-base 1×</option><option value="source">Fonte subdividida completa</option></select></label><label>Candidato <select id="mode"><option value="stable">Compilador com refit opcional</option><option value="old">Compilador anterior</option></select></label><label><input id="fit" type="checkbox" checked>Visão geral</label>
<p id="outcome"></p><p id="scale" class="note"></p><section><figure><figcaption id="referenceTitle"></figcaption><div class="view"><img id="source" alt="Referência geométrica"></div></figure><figure><figcaption id="candidateTitle"></figcaption><div class="view"><img id="candidate" alt="Geometria compilada"></div></figure></section><p id="metric"></p>
<h2>Tempo total e GPU</h2><p class="note">Medianas de16 lotes/caminho,128 frames/lote, oito rodadas alternadas com duplicatas. Tempos em ms/frame de lote; incluem codificação/submissão/espera e não representam FPS de gameplay. Câmera e luz variam. O painel mostra capturas offscreen, não um renderer em tempo real.</p><table><thead><tr><th>Densidade</th><th>GPU fonte</th><th>GPU anterior</th><th>GPU candidato</th><th>Total fonte</th><th>Total anterior</th><th>Total candidato</th><th>Total / pedra-base</th><th>Duplicatas candidato</th></tr></thead><tbody id="timings"></tbody></table>
<h2>Fidelidade e compilação</h2><p class="note">Gates mantidos:RMSE≤0,005,p99≤0,02,cobertura≤0,0005. Passar não significa pixels idênticos. A própria rasterização da subdivisão falha em quatro vistas distantes; consultar ambas as referências. O candidato não recebe a malha parental. Refits que não reduzem triângulos e bytes conservam o anterior; escolha offline, sem tabela por pose.</p><table><thead><tr><th>Fator</th><th>Triângulos anterior → escolhido</th><th>Refit selecionado</th><th>Falhas vs fonte / base</th><th>Buffer proxy escolhido</th><th>Build anterior + refit</th></tr></thead><tbody id="qualityTable"></tbody></table>
<p><a href="summary.json">Resumo</a> · <a href="results.json">Medições brutas</a> · <a href="ray-diagnostics.json">Diagnóstico por raios dos pixels divergentes</a> · <a href="https://polyhaven.com/a/boulder_01">Boulder01</a>, Rico Cilliers, CC0. Powered by Poly Haven. Geometria/normais importadas, material diagnóstico compartilhado.</p>
<script>const data=__DATA__;const $=id=>document.getElementById(id),distances=[2.7,64];
function update(){const f=+$('factor').value,w=+$('resolution').value,d=+$('distance').value,frame=+$('frame').value*2,mode=$('mode').value,ref=$('reference').value;const row=data.rows.find(x=>x.factor===f&&x.width===w&&Math.abs(x.distance-distances[d])<.001&&x.frame===frame),q=row[`${mode}_${ref}`];$('source').src=`${f}-${w}-${d}-${frame}-${ref}.png`;$('candidate').src=`${f}-${w}-${d}-${frame}-${mode}.png`;$('referenceTitle').textContent=$('reference').selectedOptions[0].text;$('candidateTitle').textContent=$('mode').selectedOptions[0].text;for(const id of ['source','candidate'])$(id).style.width=$('fit').checked?'100%':`${w/devicePixelRatio}px`;$('scale').textContent=$('fit').checked?'Visão geral reduzida. Desative para escala física nominal.':`1 pixel renderizado por pixel físico nominal (DPR ${devicePixelRatio}); zoom100%. Role para explorar.`;const s=data.summary.quality.find(x=>x.factor===f);$('outcome').textContent=`Contra pedra-base: ${s.stable_base.failures}/36 falhas. Contra fonte subdividida: ${s.stable_source.failures}/36. A fonte muda em ${s.density.failures}/36 controles.`;$('metric').textContent=`Entrada ${row.source_triangles} T; anterior ${row.old_triangles} T; escolhido ${row.triangles} T\nRMSE ${q.rmse.toFixed(8)}; p99 ${q.p99.toFixed(8)}; máximo ${q.max.toFixed(6)}; cobertura ${q.coverage_error.toFixed(6)}; ${q.pass?'PASSOU os limites':'FALHOU'}\nLimite geométrico projetado ${row.projected_error.toFixed(6)} px.`;$('timings').innerHTML=data.summary.cohorts.filter(c=>c.width===w&&Math.abs(c.distance-distances[d])<.001).map(c=>`<tr><td>${c.factor}×</td>${['source','old','stable'].map(m=>`<td>${c[m].gpu_ms.toFixed(5)}</td>`).join('')}${['source','old','stable'].map(m=>`<td>${c[m].total_batch_ms_per_frame.toFixed(5)}</td>`).join('')}<td>${(c.stable.total_batch_ms_per_frame/c.base.total_batch_ms_per_frame).toFixed(3)}×</td><td>${c.stable.duplicate_wall_percent.toFixed(1)}%</td></tr>`).join('');}
for(const id of ['factor','resolution','distance','frame','mode','reference','fit'])$(id).addEventListener('input',update);window.addEventListener('resize',update);for(const id of ['source','candidate'])$(id).onload=()=>{if(!$('fit').checked){const e=$(id),v=e.parentElement;v.scrollLeft=(e.clientWidth-v.clientWidth)/2;v.scrollTop=(e.clientHeight-v.clientHeight)/2;}};
$('qualityTable').innerHTML=data.summary.quality.map(q=>{const b=data.summary.builds[q.factor],chosen=b[1].selected?b[1]:b[0];return `<tr><td>${q.factor}×</td><td>${q.old_triangles} → ${q.triangles}</td><td>${b[1].selected?'sim':'anterior mantido'}</td><td>${q.stable_source.failures} / ${q.stable_base.failures}</td><td>${(chosen.gpu_bytes/1048576).toFixed(3)} MiB</td><td>${((b[0].compile_ms+b[1].compile_ms)/1000).toFixed(3)}s</td></tr>`}).join('');update();</script></body></html>'''
    if BORDERS:
        html = html.replace('Recuperação estável da pedra', 'Bordas assimétricas da pedra')
        html = html.replace('O compilador recalcula o plano e o campo de normais usando pontos mais afastados da própria região. Todas as faces são revalidadas sob os mesmos limites.', 'O compilador permite reunir caminhos de borda colineares quando um lado divide uma aresta e o outro não. Erros acumulados, validação de regiões e restauração de bordas nativas são preservados.')
        html = html.replace('anterior e refit', 'pacote anterior e correção de bordas')
        html = html.replace('Compilador com refit opcional', 'Correção de bordas opcional')
        html = html.replace('Compilador anterior</option>', 'Melhor pacote anterior (inclui refit)</option>')
        html = html.replace('Refits que não reduzem', 'Candidatos que não reduzem')
        html = html.replace('Refit selecionado', 'Bordas selecionadas')
        html = html.replace('Build anterior + refit', 'Build candidato')
        html = html.replace(' · <a href="ray-diagnostics.json">Diagnóstico por raios dos pixels divergentes</a>', '')
    if EXTERNAL:
        html = html.replace('Bordas assimétricas da pedra', 'Compilação de uma segunda malha externa')
        html = html.replace('pedra-base', 'malha-base').replace('Pedra-base', 'Malha-base').replace('pedra', 'malha')
        html = html.replace('A própria rasterização da subdivisão falha em quatro vistas distantes; consultar ambas as referências.', 'Consulte ambas as referências e as falhas registradas, inclusive quando os tempos melhorarem.')
        html = html.replace('https://polyhaven.com/a/boulder_01', 'https://polyhaven.com/a/wooden_stool_02').replace('Boulder01', 'Wooden Stool02').replace('Rico Cilliers', 'Kuutti Siitonen')
        html = html.replace('Build candidato', 'Build anterior + candidato')
        html = html.replace('<h1>', '<p>Segunda malha externa: banco com pernas e vãos, dois primitives e um material. Posições/normais importadas integralmente; UV/texturas/PBR originais não testados.</p><h1>',1)
        html = html.replace('Limite geométrico projetado ${row.projected_error.toFixed(6)} px.', 'Limite selecionado ${row.projected_error.toFixed(6)} px; candidato ${row.candidate_projected_error.toFixed(6)} px. Desenhados ${row.drawn_triangles} T; fallback ${row.fallback?"sim":"não"}.')
        html = html.replace('<th>Duplicatas candidato</th>', '<th>Duplicatas candidato</th><th>Frames usando proxy / 128</th>')
        html = html.replace('${c.stable.duplicate_wall_percent.toFixed(1)}%</td></tr>', '${c.stable.duplicate_wall_percent.toFixed(1)}%</td><td>${c.stable.proxy_frames}</td></tr>')
        html = html.replace('A escolha entre pacote anterior e correção de bordas ocorre no build; o frame desenha o pacote pronto.', 'A escolha de pacote ocorre no build. Em cada frame, o certificado projetado decide entre proxy e fonte completa, com custo incluído nos tempos; fallback aparece nos contadores.')
        integration=json.loads((OUT/'integrated.json').read_text())
        table='<h2>Referência com amostragem dentro do pixel</h2><p>ROI de64×64 ou128×128 pixels dentro dos viewports1728×1117 ou3456×2234, câmera distante. MédiasRGBA lineares de1/4/16/64amostras.64amostras não são integral exata. Convergência mostra a diferença16→64 da fonte; essas capturas não entram nos tempos acima. Branco/preto de contorno têm cobertura fracionária, preservada na média.</p><table><tr><th>Asset / fator / pose</th><th>Fonte vs base1spp RMSE</th><th>Fonte vs base64spp RMSE</th><th>Compilado vs base64spp RMSE</th><th>Convergência fonte16→64 RMSE</th><th>Comparação ampliada</th></tr>'
        for row in integration['rows']:
            if row['samples']!=64: continue
            single=next(x for x in integration['rows'] if x['asset']==row['asset'] and x['factor']==row['factor'] and x['frame']==row['frame'] and x['samples']==1)
            stem=f"integrated-{row['asset']}-{row['factor']}-{row['frame']}-8"
            table+=f"<tr><td>{row['asset']} / {row['factor']}× / {row['frame']} / {row['width']}px</td><td>{single['source_base']['rmse']:.8f}</td><td>{row['source_base']['rmse']:.8f}</td><td>{row['chosen_base']['rmse']:.8f}</td><td>{row['convergence_base_source_chosen'][1]['rmse']:.8f}</td><td>"+''.join(f'<a href="{stem}-{m}.png">{m}</a> ' for m in ['base','source','chosen'])+'</td></tr>'
        table+='</table><p>Auditoria100× perto: limite global máximo0,17344px; máximo regional0,06735px. O seletor medido ainda usa o global e faz fallback. O cálculo regional é diagnóstico offline, não ganho de runtime implementado. <a href="certificate-audit.json">Certificados por pose</a></p><p><a href="integrated.json">Amostras e convergência</a> · <a href="import.json">Proveniência da malha</a></p>'
        html=html.replace('<script>',table+'<script>',1)
    (OUT/'index.html').write_text(html.replace('__DATA__',json.dumps({'summary':summary,'rows':data['quality']})))
    print(json.dumps(summary,indent=2))
if __name__=='__main__':
    main()
