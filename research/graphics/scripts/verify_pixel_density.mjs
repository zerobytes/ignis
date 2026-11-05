// Execute shipped UI and validate physical crops. This is not a native browser.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
const root='artifacts/appearance-probe/pixel-density';
const html=fs.readFileSync(path.join(root,'index.html'),'utf8');
let draws=0,loads=0,interval=null,resize;
class Element {
  constructor(){this.value='0';this.children=[];this.textContent='';this.style={}}
  append(...x){this.children.push(...x)}
  replaceChildren(...x){this.children=x}
  getBoundingClientRect(){return {left:0,top:0,width:700,height:450}}
  getContext(){return {drawImage:(img,...args)=>{draws++;if(args.length===8){const [x,y,w,h,dx,dy,dw,dh]=args;assert(args.every(Number.isFinite));assert(x>=0&&y>=0&&x+w<=img.width&&y+h<=img.height);assert.equal(dw,512);assert.equal(dh,512);assert.equal(dx,0);assert.equal(dy,0)}}}}
}
const ids=Object.fromEntries([...html.matchAll(/id="([^"]+)"/g)].map(m=>[m[1],new Element()]));
ids.algorithm.value='2';ids.density.value='3';ids.metric.value='total_ms';ids.zoom.value='1';ids.frame.value='1';
let pending=[],defer=false;
class Image {
 set src(file){loads++;const p=path.join(root,file);const finish=()=>{if(!fs.existsSync(p)){this.onerror();return}const b=fs.readFileSync(p);this.width=b.readUInt32BE(16);this.height=b.readUInt32BE(20);this.onload()};if(defer)pending.push(finish);else queueMicrotask(finish)}
}
const window={devicePixelRatio:2,addEventListener:(type,fn)=>{assert.equal(type,'resize');resize=fn}};
const context=vm.createContext({document:{getElementById:id=>ids[id],createElement:()=>new Element()},Image,window,console,setInterval:fn=>(interval=fn,1),clearInterval:()=>interval=null});
vm.runInContext(html.match(/<script>([\s\S]*?)<\/script>/)[1],context);
const flush=async()=>{for(let i=0;i<3;i++)await new Promise(r=>setImmediate(r))};
await flush();let cases=0;
for(const resolution of ['0','1'])for(const density of ['0','1','2','3'])for(const algorithm of ['1','2']){
 ids.resolution.value=resolution;ids.density.value=density;ids.algorithm.value=algorithm;
 for(let frame=0;frame<5;frame++){
  ids.frame.value=String(frame);ids.algorithm.onchange();await flush();
  assert.equal(ids.status.textContent,'');assert.equal(ids.overview.children.length,2);assert.equal(ids.crops.children.length,2);
  assert(ids.quality.textContent.includes('critério experimental'));assert.equal(ids.growth.children.length,4);
  assert(ids.counts.textContent.includes('Triângulos:'));assert(ids.pose.textContent.includes(`posição ${frame+1}/5`));
  for(const id of ['chart','plateau']){assert(ids[id].innerHTML.includes('<polyline'));assert(!/NaN|Infinity/.test(ids[id].innerHTML))}
  for(const f of ids.crops.children)assert.equal(f.children[1].style.width,'256px');cases++;
 }
}
ids.overview.children[0].children[1].onpointerdown({clientX:699,clientY:449});
for(const dpr of [1,1.5,2,3])for(const z of [1,2,4]){window.devicePixelRatio=dpr;ids.zoom.value=String(z);resize();assert.equal(ids.crops.children[0].children[1].style.width,`${512*z/dpr}px`)}
for(const metric of ['gpu_ms','total_ms','cpu_ms']){ids.metric.value=metric;ids.metric.onchange();assert(!/NaN|Infinity/.test(ids.chart.innerHTML));assert(!/NaN|Infinity/.test(ids.plateau.innerHTML))}
ids.frame.value='4';ids.play.onclick();assert(interval);interval();await flush();assert.equal(ids.frame.value,'0');ids.play.onclick();assert.equal(interval,null);
// Resolve the old frame AFTER the new one to exercise the epoch guard.
defer=true;ids.frame.value='1';ids.frame.oninput();assert.equal(ids.overview.children.length,0);
ids.frame.value='2';ids.frame.oninput();const old=pending.splice(0,2),fresh=pending.splice(0);fresh.forEach(fn=>fn());await flush();const newest=ids.overview.children[0];old.forEach(fn=>fn());await flush();
assert.equal(ids.overview.children[0],newest);assert(ids.pose.textContent.includes('posição 3/5'));assert.equal(ids.status.textContent,'');
const result={cases,loads,draws,dpr_and_crops:true,playback_and_stale_load:true,native_browser:false};
fs.writeFileSync(path.join(root,'ui-verification.json'),JSON.stringify(result,null,2)+'\n');console.log(result);
