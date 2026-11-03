/** EAF-189 image-only reconstruction study. Source dimensions are hypotheses.
 * No imported mesh, image texture, segmentation model or generated 3D asset.
 */
import * as THREE from 'three';
import {buildTaperedSweepGeometry} from '../vendor/img2threejs/tapered_sweep';

type Section = [number,number,number,number]; // y, half width, half depth, centre z
export const sections: Section[] = [
 [.115,.27,.67,.37],[.16,.29,.69,.37],[.24,.30,.65,.35],
 [.34,.29,.54,.28],[.46,.255,.39,.17],[.60,.215,.265,.035],
 [.78,.20,.235,0],[1.00,.21,.23,-.01],[1.30,.23,.245,-.015],
 [1.65,.245,.255,-.01],[2,.26,.265,0],[2.40,.275,.275,.01],
 [2.80,.285,.28,.02],[3.15,.295,.29,.025],
];
const lerp=THREE.MathUtils.lerp;
function sectionAt(y:number,coarse=false):number[]{
 const source=coarse?[[.115,.32,.50,.2],[.5,.27,.34,.1],[.8,.22,.25,0],[3.15,.30,.29,.025]]:sections;
 let i=0;while(i<source.length-2&&source[i+1][0]<y)i++;
 const a=source[i],b=source[i+1],t=THREE.MathUtils.clamp((y-a[0])/(b[0]-a[0]),0,1);
 return [lerp(a[1],b[1],t),lerp(a[2],b[2],t),lerp(a[3],b[3],t)];
}
export function surface(y:number,angle:number,offset=0,detail=true,coarse=false):THREE.Vector3{
 const [w,d,z]=sectionAt(y,coarse);
 const fold=detail?(Math.exp(-(((y-.93)/.18)**2))*.014*Math.sin((y-.93)*24+angle*3)
   +Math.exp(-(((y-.66)/.13)**2))*.009*Math.cos(y*34-angle*2)):0;
 const yy=y+(y>2.8?(y-2.8)/.35*(-.07*Math.cos(angle)+.025*Math.sin(angle)):0);
 return new THREE.Vector3(Math.sin(angle)*(w+offset+fold),yy,z+Math.cos(angle)*(d+offset+fold));
}

function upper(coarse:boolean):THREE.BufferGeometry{
 const radial=64, rings=85,pos:number[]=[],indices:number[]=[],uv:number[]=[];
 for(let i=0;i<rings;i++)for(let j=0;j<=radial;j++){
   const a=(j===radial?0:j)*Math.PI*2/radial,y=lerp(.115,3.15,i/(rings-1));
   const p=surface(y,a,0,!coarse,coarse);pos.push(...p.toArray());uv.push(j/radial,i/(rings-1));
 }
 for(let i=0;i<rings-1;i++)for(let j=0;j<radial;j++){
   const a=i*(radial+1)+j,b=a+radial+1;
   indices.push(a,a+1,b,a+1,b+1,b);
 }
 // A real opening: inner collar returns into the boot, with a visible rim.
 const inner=pos.length/3;
 for(const y of [3.15,2.94])for(let j=0;j<=radial;j++){
   const p=surface(y,(j===radial?0:j)*Math.PI*2/radial,-.021,!coarse,coarse);
   pos.push(...p.toArray());uv.push(j/radial,y/3.15);
 }
 const outer=(rings-1)*(radial+1);
 for(let j=0;j<radial;j++){
   indices.push(outer+j,outer+j+1,inner+j,outer+j+1,inner+j+1,inner+j);
   indices.push(inner+j,inner+j+1,inner+radial+1+j,inner+j+1,inner+radial+2+j,inner+radial+1+j);
 }
 const base=pos.length/3;pos.push(0,.115,.27);uv.push(.5,0);
 for(let j=0;j<radial;j++)indices.push(base,j+1,j);
 const g=new THREE.BufferGeometry();g.setAttribute('position',new THREE.Float32BufferAttribute(pos,3));
 g.setAttribute('uv',new THREE.Float32BufferAttribute(uv,2));g.setIndex(indices);g.computeVertexNormals();
 return g;
}

function soleGeometry():THREE.BufferGeometry{
 // Explicit horizontal rings: grounded sole, not a stretched sphere.
 const points:number[]=[],indices:number[]=[],n=64;
 for(const [y,scale] of [[.035,1.01],[.065,1.05],[.115,1.045],[.135,1.00]]){
   for(let j=0;j<n;j++){
     const a=j/n*Math.PI*2;const z=.37+Math.cos(a)*.695*scale;
     const lift=.085*(1-THREE.MathUtils.smoothstep(z,-.01,.40));
     const yy=y<=.065?y+lift:y;
     points.push(Math.sin(a)*.295*scale,yy,z);
   }
 }
 for(let k=0;k<3;k++)for(let j=0;j<n;j++){
   const a=k*n+j,b=k*n+(j+1)%n,c=(k+1)*n+j,d=(k+1)*n+(j+1)%n;
   indices.push(a,b,c,b,d,c);
 }
 for(const k of [0,3]){
   const c=points.length/3;points.push(0,k===0?.035:.135,.27);
   for(let j=0;j<n;j++)k===0?indices.push(c,k*n+(j+1)%n,k*n+j):indices.push(c,k*n+j,k*n+(j+1)%n);
 }
 const g=new THREE.BufferGeometry();g.setAttribute('position',new THREE.Float32BufferAttribute(points,3));g.setIndex(indices);g.computeVertexNormals();return g;
}

function pathMesh(points:THREE.Vector3[],width:number,thickness:number,material:THREE.Material,name:string):THREE.Mesh{
 const mesh=new THREE.Mesh(buildTaperedSweepGeometry({stations:points.map((p,i)=>({
   position:p.toArray() as [number,number,number],rx:thickness,rz:width*(.85+.15*Math.sin(i/points.length*Math.PI))
 })),radialSegments:8,capEnds:true}),material);mesh.name=name;return mesh;
}

function leaf(y:number,a:number,dy:number,da:number,material:THREE.Material,name:string):THREE.Mesh{
 const stations=[];
 for(let i=0;i<=8;i++){
   const t=i/8,p=surface(y+dy*t,a+da*t,0);
   stations.push({position:p.toArray() as [number,number,number],rx:i===0||i===8?0:.0015,
     rz:i===0||i===8?0:Math.sin(Math.PI*t)**.85*.029});
 }
 const m=new THREE.Mesh(buildTaperedSweepGeometry({stations,radialSegments:6,capEnds:true}),material);
 m.name=name;return m;
}

export function createBoot(stage='finish',neutral=false):THREE.Group{
 const group=new THREE.Group();group.name='B0';
 const mat=(color:number,roughness:number,metalness=0)=>new THREE.MeshStandardMaterial({color,roughness,metalness});
 const clay=mat(0x99999a,.8),leather=mat(0x242b23,.56,.015),gold=mat(0xb5a071,.65,.2),sole=mat(0x382b20,.68);
 const mesh=(g:THREE.BufferGeometry,m:THREE.Material,name:string)=>{const o=new THREE.Mesh(g,neutral?clay:m);o.name=name;o.castShadow=true;o.receiveShadow=true;group.add(o);return o;};
 mesh(upper(stage==='volume'),leather,'B1-upper');mesh(soleGeometry(),sole,'B2a-sole');
 const heel=new THREE.BoxGeometry(.44,.105,.31,1,1,1);heel.translate(0,.075,-.16);
 mesh(heel,sole,'B2b-heel');
 if(stage==='volume')return group;
 // Border of collar and the overlapping panel follow the 3D surface.
 const collar=[];for(let i=0;i<=96;i++)collar.push(surface(3.15,i/96*Math.PI*2,.003));
 group.add(pathMesh(collar,.004,.003,neutral?clay:gold,'B1b-collar-trim'));
 const v=[];for(let i=0;i<=50;i++){const a=lerp(-1.32,1.15,i/50),y=1.02-.23*Math.exp(-((a/.55)**2));v.push(surface(y,a,.001));}
 group.add(pathMesh(v,.004,.003,neutral?clay:gold,'B1f-panel-trim'));
 const lower=[];for(let i=0;i<=35;i++){const a=lerp(-1.45,1.15,i/35);lower.push(surface(.48+.11*Math.sin(a+.5),a,.001));}
 group.add(pathMesh(lower,.003,.0025,neutral?clay:sole,'B3d-vamp-seam'));
 if(stage!=='finish')return group;
 const branches=[
   {start:.80,end:3.02,angle:.18,drift:.48,count:9},
   {start:1.45,end:2.94,angle:.80,drift:.34,count:7},
   {start:.42,end:1.30,angle:.10,drift:.40,count:5},
 ];
 branches.forEach((b,k)=>{
   const points=[];for(let i=0;i<=48;i++){const t=i/48;points.push(surface(lerp(b.start,b.end,t),b.angle+b.drift*Math.sin(t*2.8+k),.001));}
   group.add(pathMesh(points,.004,.003,neutral?clay:gold,'B3a-branch-'+k));
   for(let j=0;j<b.count;j++){
     const t=(j+.35)/b.count,y=lerp(b.start,b.end,t),a=b.angle+b.drift*Math.sin(t*2.8+k),side=j%2?1:-1;
     group.add(leaf(y,a,.23+.07*Math.sin(j*1.3),side*(.64+.15*Math.cos(j)),neutral?clay:gold,'B3b-leaf-'+k+'-'+j));
     if(j%3===1){const twig=[];for(let i=0;i<=8;i++)twig.push(surface(y+i/8*.12,a+side*i/8*.38,.001));group.add(pathMesh(twig,.002,.002,neutral?clay:gold,'B3a-twig-'+k+'-'+j));}
   }
 });
 // Scattered short finishing lines on the toe; source is sparse/ambiguous.
 for(let k=0;k<3;k++){
   const line=[];for(let i=0;i<=14;i++)line.push(surface(.22+k*.035,lerp(-.42,.27,i/14),.001));
   group.add(pathMesh(line,.0015,.0015,neutral?clay:gold,'B3c-toe-line-'+k));
 }
 return group;
}
