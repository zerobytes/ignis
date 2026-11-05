// Exercise the shipped JS with a mocked DOM. This is not native browser acceptance.
import fs from 'node:fs';import path from 'node:path';import vm from 'node:vm';import assert from 'node:assert/strict';
const root='artifacts/appearance-probe/adaptive-assets';
const html=fs.readFileSync(path.join(root,'index.html'),'utf8');
class Element{constructor(){this.value='0';this.textContent='';this.children=[];this.style={}}append(...x){this.children.push(...x)}replaceChildren(...x){this.children=x}}
const ids=Object.fromEntries([...html.matchAll(/id="([^"]+)"/g)].map(m=>[m[1],new Element()]));
ids.asset.value='rock';ids.zoom.value='1';
let interval=null,resize,loads=0;
class Image extends Element{set src(file){loads++;queueMicrotask(()=>{if(fs.existsSync(path.join(root,file)))this.onload();else this.onerror()})}}
const window={devicePixelRatio:2,addEventListener:(type,fn)=>{assert.equal(type,'resize');resize=fn}};
const context=vm.createContext({document:{getElementById:id=>ids[id],createElement:()=>new Element()},Image,window,console,setInterval:fn=>(interval=fn,1),clearInterval:()=>interval=null});
vm.runInContext(html.match(/<script>([\s\S]*?)<\/script>/)[1],context);
const flush=async()=>{for(let i=0;i<3;i++)await new Promise(r=>setImmediate(r))};await flush();let cases=0;
for(const asset of ['rock','chest','tree'])for(const resolution of ['0','1']){
 ids.asset.value=asset;ids.resolution.value=resolution;ids.asset.onchange();await flush();
 for(let frame=0;frame<=Number(ids.frame.max);frame++){
  ids.frame.value=String(frame);ids.frame.oninput();await flush();assert.equal(ids.status.textContent,'');assert.equal(ids.images.children.length,4);
  for(const fig of ids.images.children){assert.equal(fig.children[1].style.width,'256px');assert.equal(fig.children[1].style.height,'256px')}
  assert(ids.display.textContent.includes('DPR do navegador 2'));assert(ids.quality.textContent.includes('RMSE'));cases++;
 }
 ids.transition.onclick();await flush();assert(ids.decision.textContent.startsWith('Resumo'));
}
for(const dpr of [1,1.5,2,3])for(const zoom of [1,2,4]){window.devicePixelRatio=dpr;ids.zoom.value=String(zoom);resize();await flush();assert.equal(ids.images.children[0].children[1].style.width,`${512*zoom/dpr}px`)}
ids.asset.value='rock';ids.asset.onchange();ids.play.onclick();assert(interval);interval();await flush();assert.equal(ids.frame.value,'1');ids.play.onclick();assert.equal(interval,null);
ids.frame.value=ids.frame.max;ids.play.onclick();interval();await flush();assert.equal(ids.frame.value,'0');ids.play.onclick();
ids.asset.value='rock';ids.asset.onchange();ids.asset.value='tree';ids.asset.onchange();await flush();assert(ids.pose.textContent.includes('Árvore'));
console.log(JSON.stringify({cases,loads,dpr_zoom_resize:true,playback_and_stale_load:true,native_browser:false}));
