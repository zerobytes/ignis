// DOM harness only: controls/capture paths/DPR crops. Not native browser acceptance.
import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
const root='artifacts/appearance-probe/geometric-field';
const html=fs.readFileSync(root+'/index.html','utf8');
const code=html.match(/<script>([\s\S]*?)<\/script>/)[1];
const elements=new Map();const draws=[];
class Element {
 constructor(id){this.id=id;this.style={};this.options=[];this._value=id==='frame'?'0':id==='candidate'?'auto':'';this.complete=true;this.clientWidth=320;this.naturalWidth=3456;this.naturalHeight=2234;}
 get value(){return this._value} set value(v){this._value=String(v)}
 replaceChildren(...children){this.options=children;if(children.length)this.value=children[0].value;}
 set src(v){assert(fs.existsSync(root+'/'+v),v);this._src=v;const b=fs.readFileSync(root+'/'+v).subarray(0,24);this.naturalWidth=b.readUInt32BE(16);this.naturalHeight=b.readUInt32BE(20);}
 get src(){return this._src}
 getContext(){return {clearRect(){},drawImage(...args){draws.push(args)}};}
}
const $=id=>{if(!elements.has(id))elements.set(id,new Element(id));return elements.get(id)};
const context=vm.createContext({document:{getElementById:$,createElement:()=>new Element('option')},window:{devicePixelRatio:2},setInterval:()=>1,clearInterval(){},console});
vm.runInContext(code,context);await vm.runInContext('render()',context);
const rows=JSON.parse(fs.readFileSync(root+'/quality.json','utf8'));let count=0;
for(const row of rows){
 for(const mode of ['auto','forced']){
  $('density').value=row.density;$('resolution').value=row.resolution;$('distance').value=row.distance;
  vm.runInContext('scenes()',context);$('scene').value=row.scene;$('frame').value=row.frame;$('candidate').value=mode;
  assert.equal(vm.runInContext('current().id',context),row.id);
  await vm.runInContext('render()',context);
  assert.equal($('a').src,row.id+'_0.png');assert.equal($('b').src,row.id+'_135.png');
  assert.equal($('c').src,row.id+`_${mode==='forced'||row.symbol_selected?325:135}.png`);
  const q=row.checks[mode==='forced'?1:2];assert($('metrics').textContent.includes(q.foreground_rmse.toFixed(6)));
  assert.equal($('ca').width,640);assert.equal($('ca').height,360);count++;
 }
}
for(const dpr of [1,1.5,2,3]){context.window.devicePixelRatio=dpr;vm.runInContext('crop()',context);assert.equal($('ca').width,Math.round(320*dpr));for(const a of draws.slice(-3)){assert.equal(a[3],a[7]);assert.equal(a[4],a[8]);}}
$('play').onclick();assert.equal($('play').textContent,'Pausar');$('play').onclick();assert.equal($('play').textContent,'Reproduzir capturas');
fs.writeFileSync(root+'/ui-verification.json',JSON.stringify({cases:count,dpr:[1,1.5,2,3],physical_crop:true,play_controls:true,native_browser:false},null,2)+'\n');
console.log(`PASS ${count} control states; physical crops at four DPRs; DOM harness only`);
