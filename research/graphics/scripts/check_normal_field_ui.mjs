// Exercise generated controls without launching or claiming a native browser.
import fs from 'node:fs';
import vm from 'node:vm';
import path from 'node:path';
const root=path.resolve(import.meta.dirname,'../artifacts/appearance-probe/normal-field');
const html=fs.readFileSync(path.join(root,'index.html'),'utf8');
const code=html.match(/<script>([\s\S]*?)<\/script>/)[1];
let count=0;
for(const dpr of [1,2]) {
 const elements={};
 for(const id of ['variant','resolution','distance','frame','mode','fit','source','candidate','summary','dvalue','candidateTitle','scale','metric','animate','qualityTable','timings']) {
  elements[id]={value:({variant:'boulder-64',resolution:'1728',distance:'2',frame:'1',mode:'field'})[id]??'',checked:id==='fit',style:{},selectedOptions:[{text:'test'}],addEventListener(){},clientWidth:1000,clientHeight:800,parentElement:{clientWidth:600,clientHeight:400}};
 }
 const context={document:{getElementById:id=>elements[id],body:{classList:{toggle(){}}}},window:{addEventListener(){}},devicePixelRatio:dpr,setInterval(){return 1},clearInterval(){}};
 vm.createContext(context);vm.runInContext(code,context);
 for(const variant of ['boulder-32','boulder-64','boulder-128'])for(const width of [1728,3456])for(let distance=0;distance<4;distance++)for(let frame=0;frame<4;frame++)for(const mode of ['field','reduced'])for(const fit of [false,true]) {
  for(const [id,value] of Object.entries({variant,resolution:width,distance,frame,mode}))elements[id].value=value;
  elements.fit.checked=fit;vm.runInContext('update()',context);
  for(const id of ['source','candidate']) {
   const image=elements[id];if(!fs.existsSync(path.join(root,image.src)))throw Error(image.src);
   if(image.style.width!==(fit?'100%':`${width/dpr}px`))throw Error('Incorrect physical pixel scale');
   image.onload();if(!fit&&image.parentElement.scrollTop!==200)throw Error('Native-size image not centered');
  }
  if(!elements.metric.textContent.includes('triângulos'))throw Error('Missing metrics');count++;
 }
 elements.animate.onclick();elements.animate.onclick();
}
console.log(`${count} simulated DOM/DPR states passed; no native browser acceptance claimed`);
