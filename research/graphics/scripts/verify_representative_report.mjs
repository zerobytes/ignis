// Run the shipped report script with a minimal DOM; native browser check is separate.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
const root='artifacts/appearance-probe/assets';
const html=fs.readFileSync(path.join(root,'index.html'),'utf8');
let loads=0,draws=0,interval;
class Element {
 constructor(){this.value='0';this.textContent='';this.children=[];}
 append(...x){this.children.push(...x)} replaceChildren(...x){this.children=x}
 getContext(){return {drawImage:(_image,x,y,w,h,dx,dy,dw,dh)=>{assert([x,y,w,h,dx,dy,dw,dh].every(Number.isFinite));assert(x>=0&&y>=0&&x+w<=256&&y+h<=256);draws++}}}
}
const ids=Object.fromEntries([...html.matchAll(/id="([^"]+)"/g)].map(m=>[m[1],new Element()]));
Object.assign(ids.asset,{value:'rock'});ids.scene.value='both';ids.sampling.value='native';ids.zoom.value='1';
class Image {set src(file){loads++;queueMicrotask(()=>fs.existsSync(path.join(root,file))?this.onload():this.onerror())}}
const context=vm.createContext({document:{getElementById:id=>ids[id],createElement:()=>new Element()},Image,setInterval:fn=>(interval=fn,1),clearInterval:()=>interval=null,console});
vm.runInContext(html.match(/<script>([\s\S]*?)<\/script>/)[1],context);
const flush=async()=>{for(let i=0;i<3;i++)await new Promise(r=>setImmediate(r))};await flush();let cases=0;
for(const asset of ['rock','chest','tree']) for(const scene of ['camera','light','both','near','zoom','seam']) for(const sampling of ['native','integrated']) {
 ids.asset.value=asset;ids.scene.value=scene;ids.sampling.value=sampling;ids.asset.onchange();await flush();
 for(let frame=0;frame<=Number(ids.frame.max);frame++){
  ids.frame.value=String(frame);ids.frame.oninput();await flush();assert.equal(ids.status.textContent,'');assert.equal(ids.images.children.length,4);assert.equal(ids.errors.children.length,2);assert(ids.quality.textContent.includes('RMSE'));assert(ids.timing.textContent.includes('Controle'));cases++;
 }
}
ids.frame.value='0';ids.back.onclick();await flush();assert.equal(Number(ids.frame.value),Number(ids.frame.max));ids.next.onclick();await flush();assert.equal(Number(ids.frame.value),0);
ids.play.onclick();assert(interval);interval();await flush();assert.equal(Number(ids.frame.value),1);ids.play.onclick();assert.equal(interval,null);
ids.zoom.value='2';ids.zoom.onchange();await flush();assert.equal(ids.status.textContent,'');
ids.asset.value='rock';ids.asset.onchange();ids.asset.value='tree';ids.asset.onchange();await flush();assert(ids.pose.textContent.includes('Árvore'));
console.log(JSON.stringify({cases,loads,draws,playback_wrap_zoom_stale_load:true,native_browser:false}));
