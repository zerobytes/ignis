const fs=require('fs'),vm=require('vm'),path=require('path');
const file=path.resolve(__dirname,'../artifacts/house-e02-rebuild/orbit.html');
const html=fs.readFileSync(file,'utf8'),blocks=[...html.matchAll(/<script[^>]*>([\s\S]*?)<\/script>/g)].map(m=>m[1]);
const elements=new Map();function el(id){if(!elements.has(id))elements.set(id,{textContent:id==='scene'?blocks[0]:'',clientHeight:600,getContext:()=>null,setAttribute(){},addEventListener(){},setPointerCapture(){}});return elements.get(id);}
const context={document:{getElementById:el},console};vm.createContext(context);vm.runInContext(blocks[1],context);
vm.runInContext(`
if(!data.vertices.every(Number.isFinite))throw Error('nonfinite geometry');
if(!data.indices.every(i=>i>=0&&i<data.vertices.length/9))throw Error('invalid index');
const v=lookAt(scene.camera.position,scene.camera.target);const t=scene.camera.target;
const projected=[v[0]*t[0]+v[4]*t[1]+v[8]*t[2]+v[12],v[1]*t[0]+v[5]*t[1]+v[9]*t[2]+v[13]];
if(projected.some(x=>Math.abs(x)>1e-5))throw Error('lookAt target not centered');
home();orbit(1000,10000);if(pitch>1.48)throw Error('orbit clamp');zoom(-100);if(distance!==2)throw Error('zoom min');zoom(100);if(distance!==180)throw Error('zoom max');pan(500,-300);if(!target.every(Number.isFinite))throw Error('pan');home();
console.log(JSON.stringify({vertices:data.vertices.length/9,triangles:data.indices.length/3,finite:true,lookAt:true,orbitZoomPan:true,webglRuntimeTested:false}));
`,context);
const reviewHtml=fs.readFileSync(path.join(path.dirname(file),'differences.html'),'utf8');
const reviewBlocks=[...reviewHtml.matchAll(/<script[^>]*>([\s\S]*?)<\/script>/g)].map(m=>m[1]);
const reviewData=JSON.parse(reviewBlocks[0]),buttons=reviewData.findings.map(f=>({dataset:{id:f.id},setAttribute(){}}));
const nodes=new Map();function node(id){if(!nodes.has(id))nodes.set(id,{textContent:id==='data'?reviewBlocks[0]:''});return nodes.get(id);}
const rc={document:{getElementById:node,querySelectorAll:()=>buttons},history:{replaceState(){}},location:{hash:''}};vm.createContext(rc);vm.runInContext(reviewBlocks[1],rc);
for(const b of buttons){b.onclick();for(const id of ['ref','render'])if(!fs.existsSync(path.join(path.dirname(file),node(id).src)))throw Error('missing review image');}
console.log(JSON.stringify({reviewSelections:buttons.length,reviewAssets:true,browserInteractionTested:false}));
