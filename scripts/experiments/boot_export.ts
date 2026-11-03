/** Export the code-built study and explicit, manually annotated landmark hypotheses. */
import * as THREE from 'three';
import {writeFileSync} from 'node:fs';
import {createBoot,surface} from './boot_model';
const root=process.argv[2];if(!root)throw new Error('output root required');
const model=createBoot('finish');model.updateMatrixWorld(true);
writeFileSync(root+'/boot.object.json',JSON.stringify(model.toJSON()));
const meshes:any[]=[];model.traverse(o=>{if(!(o instanceof THREE.Mesh))return;
 const p=o.geometry.getAttribute('position'),idx=o.geometry.getIndex(),positions=[],triangles=[];
 for(let i=0;i<p.count;i++)positions.push(new THREE.Vector3().fromBufferAttribute(p,i).applyMatrix4(o.matrixWorld).toArray());
 for(let i=0;i<(idx?.count||p.count);i+=3)triangles.push([0,1,2].map(j=>idx?idx.getX(i+j):i+j));
 meshes.push({id:o.name,positions,triangles});});
writeFileSync(root+'/boot.geometry.json',JSON.stringify(meshes));
const side=-1.055,other=side+Math.PI;
const annotations=[
 {name:'cuff-left',world:surface(3.15,side).toArray(),observed:[102*2,43*2]},
 {name:'cuff-right',world:surface(3.15,other).toArray(),observed:[174*2,67*2]},
 {name:'shaft-left',world:surface(1.85,side).toArray(),observed:[109*2,200*2]},
 {name:'shaft-right',world:surface(1.85,other).toArray(),observed:[167*2,200*2]},
 {name:'ankle-left',world:surface(.85,side).toArray(),observed:[101*2,318*2]},
 {name:'ankle-right',world:surface(.85,other).toArray(),observed:[174*2,319*2]},
 {name:'toe-extremity',world:[0,.065,.27+.555*1.05],observed:[18*2,430*2]},
 {name:'heel-back',world:[.22,.0225,-.315],observed:[174*2,412*2]},
];
writeFileSync(root+'/landmarks.json',JSON.stringify({
 reference:'reference.png',annotation_note:'Approximate manual image landmarks and hypothesized shape correspondences; pixel precision is not claimed.',
 initial_camera:{image_width:400,image_height:940,fov_degrees:29,yaw_degrees:0,pitch_degrees:0,roll_degrees:0,position:[0,1.6,8.5]},
 correspondences:annotations},null,2));
