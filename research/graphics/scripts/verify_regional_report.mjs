// Runs the shipped report JavaScript; native-browser presentation remains a separate check.
import fs from 'node:fs';import path from 'node:path';import vm from 'node:vm';import assert from 'node:assert/strict';
const root='artifacts/appearance-probe/regional-scene',html=fs.readFileSync(path.join(root,'index.html'),'utf8');
let draws=0,loads=0,interval=null,resize;
class Element{constructor(){this.value='0';this.children=[];this.textContent='';this.style={}}append(...x){this.children.push(...x)}replaceChildren(...x){this.children=x}getBoundingClientRect(){return {left:0,top:0,width:700,height:450}}getContext(){return {drawImage:(img,...args)=>{draws++;if(args.length===8){const [x,y,w,h,dx,dy,dw,dh]=args;assert([x,y,w,h,dx,dy,dw,dh].every(Number.isFinite));assert(x>=0&&y>=0&&x+w<=img.width&&y+h<=img.height);assert.equal(dw,512);assert.equal(dh,512)}}}}}
const ids=Object.fromEntries([...html.matchAll(/id="([^"]+)"/g)].map(m=>[m[1],new Element()]));ids.frame.value='6';ids.zoom.value='1';
class Image{set src(file){loads++;const p=path.join(root,file);queueMicrotask(()=>{if(!fs.existsSync(p)){this.onerror();return}const b=fs.readFileSync(p);this.width=b.readUInt32BE(16);this.height=b.readUInt32BE(20);this.onload()})}}
const window={devicePixelRatio:2,addEventListener:(type,fn)=>{assert.equal(type,'resize');resize=fn}};
const context=vm.createContext({document:{getElementById:id=>ids[id],createElement:()=>new Element()},Image,window,console,setInterval:fn=>(interval=fn,1),clearInterval:()=>interval=null});
vm.runInContext(html.match(/<script>([\s\S]*?)<\/script>/)[1],context);const flush=async()=>{for(let i=0;i<3;i++)await new Promise(r=>setImmediate(r))};await flush();let cases=0;
for(const resolution of ['0','1']){ids.resolution.value=resolution;ids.resolution.onchange();await flush();for(let frame=0;frame<=Number(ids.frame.max);frame++){ids.frame.value=String(frame);ids.frame.oninput();await flush();assert.equal(ids.status.textContent,'');assert.equal(ids.overview.children.length,2);assert([3,4].includes(ids.crops.children.length));assert(ids.counts.textContent.includes('mistura visível'));assert(ids.quality.textContent.includes('foreground'));ids.mixed.onclick();assert(ids.focus.textContent.includes('pixels de resumo'));for(const f of ids.crops.children)assert.equal(f.children[1].style.width,'256px');cases++}}
ids.overview.children[0].children[1].onpointerdown({clientX:699,clientY:449});assert(ids.focus.textContent.includes('Recorte escolhido'));
for(const dpr of [1,1.5,2,3])for(const z of [1,2,4]){window.devicePixelRatio=dpr;ids.zoom.value=String(z);resize();assert.equal(ids.crops.children[0].children[1].style.width,`${512*z/dpr}px`)}
ids.frame.value=ids.frame.max;ids.play.onclick();assert(interval);interval();await flush();assert.equal(ids.frame.value,'0');ids.play.onclick();assert.equal(interval,null);
ids.frame.value='1';ids.frame.oninput();ids.frame.value='2';ids.frame.oninput();await flush();assert(ids.pose.textContent.includes('posição 3/26'));assert.equal(ids.status.textContent,'');
console.log(JSON.stringify({cases,loads,draws,dpr_and_crops:true,playback_and_stale_load:true,native_browser:false}));
