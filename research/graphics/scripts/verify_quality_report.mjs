#!/usr/bin/env node
// Exercise the report's actual JS with a minimal DOM and real image paths.
// This validates control state/data integrity, not native browser layout or input.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
const root = path.resolve(process.argv[2] ?? 'artifacts/appearance-probe/quality');
const html = fs.readFileSync(path.join(root, 'index.html'), 'utf8');
const script = html.match(/<script>([\s\S]*?)<\/script>/)[1];
const cropCalls = [];
class Element {
  constructor(tag) { this.tag = tag; this.children = []; this.value = ''; this.checked = false; this.textContent = ''; this.innerHTML = ''; }
  append(...children) { this.children.push(...children); }
  replaceChildren(...children) { this.children = children; }
  setAttribute() {}
  getContext() { return {drawImage: (...a) => cropCalls.push(a), beginPath() {}, stroke() {}, moveTo() {}, lineTo() {}}; }
}
const ids = Object.fromEntries([...html.matchAll(/id="([^"]+)"/g)].map(m => [m[1], new Element(m[1])]));
Object.assign(ids.scene,{value:'glossy'});Object.assign(ids.quality,{value:'0'});Object.assign(ids.frame,{value:'9'});if(ids.control)Object.assign(ids.control,{value:'old'});
let imageLoads = 0, interval;
class Image {
  set src(file) { this.file = file; imageLoads++; queueMicrotask(() => fs.existsSync(path.join(root,file)) ? this.onload() : this.onerror()); }
}
const ctx = vm.createContext({document:{getElementById:id=>ids[id],createElement:tag=>new Element(tag)},Image,console,setInterval:fn=>(interval=fn,1),clearInterval:()=>{interval=null;}});
vm.runInContext(script,ctx,{filename:'quality-report.js'});
const flush = async () => { for(let i=0;i<5;i++) await new Promise(resolve=>setImmediate(resolve)); };
await flush();
let checks=0;
for(const scene of ['diffuse','glossy','grazing','transition']) {
  ids.scene.value=scene;ids.scene.onchange();await flush();
  for(const quality of [0,1,2]) for(const frame of [0,Number(ids.frame.max)/2,Number(ids.frame.max)]) {
    ids.quality.value=String(quality);ids.frame.value=String(frame);ids.frame.oninput();await flush();
    assert.equal(ids.images.children.length,4);
    assert(!ids.pose.textContent.includes('indisponível'));
    assert(ids.position.textContent.startsWith(`${frame+1}/`));
    assert.equal(ids.profile.textContent,['Mais resumos','Intermediário','Mais geometria'][quality]);checks++;
  }
}
if(ids.control) for(const control of ['old','fixed','full']) for(const errors of [false,true]) {
  ids.control.value=control;ids.errors.checked=errors;ids.zoom.checked=true;ids.pixels.checked=true;ids.control.onchange();await flush();checks++;
}
ids.zoom.checked=true;ids.pixels.checked=true;ids.zoom.onchange();await flush();
ids.frame.value='0';ids.back.onclick();await flush();assert.equal(Number(ids.frame.value),Number(ids.frame.max));
ids.next.onclick();await flush();assert.equal(Number(ids.frame.value),0);
ids.play.onclick();assert(interval);interval();await flush();assert.equal(Number(ids.frame.value),1);ids.play.onclick();assert.equal(interval,null);checks+=3;
// Stale in-flight requests must not replace the final scene/profile.
ids.scene.value='glossy';ids.scene.onchange();ids.quality.value='2';ids.quality.oninput();ids.scene.value='transition';ids.scene.onchange();await flush();
assert.equal(ids.profile.textContent,'Mais geometria');assert(ids.pose.textContent.includes('10.042'));checks++;
for(const [,x,y,w,h,dx,dy,dw,dh] of cropCalls) { assert([x,y,w,h,dx,dy,dw,dh].every(Number.isFinite));assert(x>=0&&y>=0&&x+w<=256&&y+h<=256); }
console.log(JSON.stringify({control_checks:checks,image_loads:imageLoads,canvas_crops:cropCalls.length,js_syntax:true,native_browser:false}));
