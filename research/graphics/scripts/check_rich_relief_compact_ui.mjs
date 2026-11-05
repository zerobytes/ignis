import fs from 'node:fs';
import vm from 'node:vm';
import path from 'node:path';
const root=path.resolve(import.meta.dirname,'../artifacts/appearance-probe/rich-relief');
const html=fs.readFileSync(path.join(root,'compact.html'),'utf8');
const code=html.match(/<script>([\s\S]*?)<\/script>/)[1];
const summary=JSON.parse(fs.readFileSync(path.join(root,'compact-summary.json'),'utf8'));
if(!summary.image_and_return_gates_pass&&!html.includes('REPROVADO'))throw Error('failure banner hidden');
let checks=0;
for(const dpr of [1,2]){
  const elements={};
  for(const id of ['resolution','case','frame','mode','scale','reference','candidate','caption','quality','cost','memory']){
    elements[id]={value:({resolution:1728,case:0,frame:0,mode:'geometric-grad'})[id]??'',style:{},addEventListener(){}};
  }
  const classes=new Set();
  const context={document:{getElementById:id=>elements[id],body:{classList:{contains:x=>classes.has(x),toggle(x){classes.has(x)?classes.delete(x):classes.add(x);}}}},window:{devicePixelRatio:dpr,addEventListener(){}}};
  vm.createContext(context);vm.runInContext(code,context);
  for(const width of [1728,3456])for(let c=0;c<5;c++)for(let frame=0;frame<=(c===4?32:8);frame++)for(const mode of ['geometric-grad','geometric-screen','geometric-packed'])for(const native of [false,true]){
    for(const [id,value] of Object.entries({resolution:width,case:c,frame,mode}))elements[id].value=value;
    if(classes.has('native')!==native)elements.scale.onclick();
    vm.runInContext('update()',context);
    for(const id of ['reference','candidate']){
      if(!fs.existsSync(path.join(root,elements[id].src)))throw Error(elements[id].src);
      if(elements[id].style.width!==(native?`${width/dpr}px`:'100%'))throw Error('physical pixel scale');
    }
    const pass=vm.runInContext(`data.quality.find(r=>r.width===${width}&&r.case===${c}&&r.frame===${frame})['${mode}'].pass`,context);
    if(!pass&&!elements.quality.textContent.includes('FALHA'))throw Error('image failure hidden');
    if(!elements.quality.textContent.includes(`${summary.returns[mode]}/10`))throw Error('return result hidden');
    if(!elements.quality.textContent.includes(`referência: ${summary.repetitions.implicit.unchanged}/128`))throw Error('reference repetition hidden');
    checks++;
  }
}
console.log(`${checks} simulated DOM/DPR states passed; native browser acceptance not established`);
