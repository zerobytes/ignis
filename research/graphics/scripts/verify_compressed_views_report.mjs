// Execute the actual report script with a minimal DOM; this is not native-browser acceptance.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
const root=process.argv[2]||'artifacts/appearance-probe/compression';
const html=fs.readFileSync(path.join(root,'index.html'),'utf8');
const script=html.match(/<script>([\s\S]*?)<\/script>/)[1];
let crops=[],loads=0,interval;
class Element {
 constructor(){this.value='0';this.checked=false;this.textContent='';this.children=[];}
 append(...xs){this.children.push(...xs)} replaceChildren(...xs){this.children=xs}
 getContext(){return {drawImage:(...args)=>crops.push(args)}}
}
const ids=Object.fromEntries([...html.matchAll(/id="([^"]+)"/g)].map(m=>[m[1],new Element()]));
ids.scene.value='both';ids.zoom.checked=true;
class Image {set src(file){loads++;queueMicrotask(()=>fs.existsSync(path.join(root,file))?this.onload():this.onerror())}}
const context=vm.createContext({document:{getElementById:id=>ids[id],createElement:()=>new Element()},Image,setInterval:fn=>(interval=fn,1),clearInterval:()=>interval=null,console});
vm.runInContext(script,context,{filename:'views-report.js'});
const flush=async()=>{for(let i=0;i<5;i++)await new Promise(resolve=>setImmediate(resolve))};await flush();let checks=0;
for(const scene of ['camera','light','both','near','seam']){
 ids.scene.value=scene;ids.scene.onchange();await flush();
 for(let frame=0;frame<=Number(ids.frame.max);frame++){
  ids.frame.value=String(frame);ids.frame.oninput();await flush();
  assert.equal(ids.images.children.length,4);assert(!ids.pose.textContent.includes('indisponível'));assert(ids.position.textContent.startsWith(`${frame+1}/`));assert(ids.quality.textContent.includes('Erro introduzido'));checks++;
 }
}
ids.frame.value='0';ids.back.onclick();await flush();assert.equal(Number(ids.frame.value),Number(ids.frame.max));ids.next.onclick();await flush();assert.equal(Number(ids.frame.value),0);
ids.play.onclick();assert(interval);interval();await flush();assert.equal(Number(ids.frame.value),1);ids.play.onclick();assert.equal(interval,null);checks+=3;
ids.zoom.checked=false;ids.zoom.onchange();await flush();ids.zoom.checked=true;ids.zoom.onchange();await flush();checks+=2;
ids.scene.value='camera';ids.scene.onchange();ids.scene.value='seam';ids.scene.onchange();await flush();assert(ids.position.textContent.endsWith('/17'));checks++;
for(const [,x,y,w,h,dx,dy,dw,dh] of crops){assert([x,y,w,h,dx,dy,dw,dh].every(Number.isFinite));assert(x>=0&&y>=0&&x+w<=256&&y+h<=256)}
console.log(JSON.stringify({checks,image_loads:loads,canvas_crops:crops.length,native_browser:false}));
