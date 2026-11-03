import * as THREE from 'three';
import {OrbitControls} from 'three/addons/controls/OrbitControls.js';
import {createBoot} from './boot_model';

const query=new URLSearchParams(location.search);
const stage=query.get('stage')||'finish',view=query.get('view')||'reference',neutral=query.has('neutral');
const width=400,height=940,mask=query.has('mask');
const scene=new THREE.Scene();scene.background=new THREE.Color(0x353736);
scene.add(new THREE.HemisphereLight(0xd7e1e2,0x4d473b,2.0));
for(const [pos,color,power] of [[[ -3,6,5],0xfff2dd,3.3],[[4,3,-2],0xd8e8ff,2],[[0,6,-4],0xffffff,1.6]] as const){
 const l=new THREE.DirectionalLight(color,power);l.position.set(...pos as [number,number,number]);scene.add(l);
}
const model=createBoot(stage,neutral);if(mask){scene.background=new THREE.Color(0);model.traverse(o=>{if(o instanceof THREE.Mesh)o.material=new THREE.MeshBasicMaterial({color:0xffffff,toneMapped:false});});}scene.add(model);
const floor=new THREE.Mesh(new THREE.PlaneGeometry(200,200),new THREE.MeshBasicMaterial({color:mask?0:0x353736,toneMapped:false}));floor.rotation.x=-Math.PI/2;floor.position.y=.019;scene.add(floor);
const camera=new THREE.PerspectiveCamera(29,width/height,.01,100);
const poses:any={reference:[4.2,3.5,7.4],front:[0,2,8.5],side:[-8.5,2,.1],right:[8.5,2,.1],rear:[0,2,-8.5],top:[.05,9,.1]};
camera.position.fromArray(poses[view]||poses.reference);camera.lookAt(0,1.61,.12);
if(query.has('fit')){
 const fit=await fetch('camera-fit.json').then(r=>r.json());
 const c=fit.exact_camera;camera.position.fromArray(c.position);camera.up.fromArray(c.up);camera.fov=c.fov_degrees;camera.lookAt(...c.target);camera.updateProjectionMatrix();
}
const renderer=new THREE.WebGLRenderer({antialias:true,preserveDrawingBuffer:true});renderer.setPixelRatio(1);renderer.setSize(width,height);
renderer.outputColorSpace=THREE.SRGBColorSpace;renderer.toneMapping=THREE.ACESFilmicToneMapping;renderer.toneMappingExposure=.85;
document.getElementById('render')!.append(renderer.domElement);
const controls=new OrbitControls(camera,renderer.domElement);controls.target.set(0,1.61,.12);controls.enableDamping=false;
if(query.has('fit')){const f=await fetch('camera-fit.json').then(r=>r.json());controls.target.fromArray(f.exact_camera.target);}
controls.update();
const draw=()=>renderer.render(scene,camera);controls.addEventListener('change',draw);draw();
async function download(name:string,data:string,type:string){
 const body=type==='image/png'?Uint8Array.from(atob(data.split(',')[1]),c=>c.charCodeAt(0)):data;
 const response=await fetch('capture/'+name,{method:'POST',headers:{'Content-Type':type},body});
 if(!response.ok)throw new Error('Evidence save failed: '+response.status);
 document.getElementById('save-status')!.textContent='Salvo: captures/'+name;
}
document.getElementById('save-png')!.onclick=async()=>{
 const name=`${stage}-${view}${neutral?'-neutral':''}${query.has('fit')?'-fit':''}${mask?'-mask':''}`;
 await download(name+'.png',renderer.domElement.toDataURL('image/png'),'image/png');
 await download(name+'.json',JSON.stringify((window as any).__BOOT_RESULT__,null,2),'application/json');
};
document.getElementById('save-model')!.onclick=()=>download('boot-object.json',JSON.stringify(model.toJSON()),'application/json');
const geometry=()=>{
 model.updateMatrixWorld(true);const meshes:any[]=[];model.traverse(o=>{if(!(o instanceof THREE.Mesh))return;
 const p=o.geometry.getAttribute('position'),index=o.geometry.getIndex(),positions=[],triangles=[];
 for(let i=0;i<p.count;i++)positions.push(new THREE.Vector3().fromBufferAttribute(p,i).applyMatrix4(o.matrixWorld).toArray());
 for(let i=0;i<(index?.count||p.count);i+=3)triangles.push([0,1,2].map(j=>index?index.getX(i+j):i+j));
 meshes.push({id:o.name,positions,triangles});});return meshes;
};
document.getElementById('save-geometry')!.onclick=()=>download('boot-geometry.json',JSON.stringify(geometry()),'application/json');
const bounds=new THREE.Box3().setFromObject(model),meshes=geometry();
const corners=[];for(const x of [-.25,.25])for(const y of [.2,1.5,3])for(const z of [-.2,.6]){
 const p=new THREE.Vector3(x,y,z).project(camera);corners.push({world:[x,y,z],projected:[(p.x+1)*width/2,(1-p.y)*height/2]});}
(window as any).__BOOT_RESULT__={stage,view,neutral,mask,width,height,vertices:meshes.reduce((s,m)=>s+m.positions.length,0),triangles:meshes.reduce((s,m)=>s+m.triangles.length,0),meshes:meshes.length,bounds:[bounds.min.toArray(),bounds.max.toArray()],camera:{position:camera.position.toArray(),target:controls.target.toArray(),up:camera.up.toArray(),fov_degrees:camera.fov},landmarks:corners};
document.getElementById('status')!.textContent=`${stage} · ${meshes.length} peças · ${meshes.reduce((s,m)=>s+m.triangles.length,0)} triângulos · referência geométrica: imagem`;
