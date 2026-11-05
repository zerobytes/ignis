// Exercise generated controls without launching or claiming a native browser.
import fs from 'node:fs';
import vm from 'node:vm';
import path from 'node:path';
const root=path.resolve(import.meta.dirname,'../artifacts/appearance-probe/coordinated-regions');
const html=fs.readFileSync(path.join(root,'index.html'),'utf8');
const code=html.match(/<script>([\s\S]*?)<\/script>/)[1];
let count=0;
for(const dpr of [1,2]) {
 const elements={};
 for(const id of ['factor','resolution','distance','frame','mode','fit','source','candidate','outcome','candidateTitle','scale','metric','qualityTable','timings']) {
  elements[id]={value:({factor:'100',resolution:'1728',distance:'0',frame:'0',mode:'coordinated'})[id]??'',checked:id==='fit',style:{},selectedOptions:[{text:'test'}],addEventListener(){},clientWidth:1000,clientHeight:800,parentElement:{clientWidth:600,clientHeight:400}};
 }
 const context={document:{getElementById:id=>elements[id],body:{classList:{toggle(){}}}},window:{addEventListener(){}},devicePixelRatio:dpr,setInterval(){return 1},clearInterval(){}};
 vm.createContext(context);vm.runInContext(code,context);
 for(const factor of [1,2,10,100])for(const width of [1728,3456])for(let distance=0;distance<2;distance++)for(let frame=0;frame<4;frame++)for(const mode of ['recovered','old','coordinated','geometry'])for(const fit of [false,true]) {
  for(const [id,value] of Object.entries({factor,resolution:width,distance,frame,mode}))elements[id].value=value;
  elements.fit.checked=fit;vm.runInContext('update()',context);
  for(const id of ['source','candidate']) {
   const image=elements[id];if(!fs.existsSync(path.join(root,image.src)))throw Error(image.src);
   if(image.style.width!==(fit?'100%':`${width/dpr}px`))throw Error('Incorrect physical pixel scale');
   image.onload();if(!fit&&image.parentElement.scrollTop!==200)throw Error('Native-size image not centered');
  }
  if(!elements.metric.textContent.includes('Entrada'))throw Error('Missing metrics');count++;
 }

}
console.log(`${count} simulated DOM/DPR states passed; no native browser acceptance claimed`);
