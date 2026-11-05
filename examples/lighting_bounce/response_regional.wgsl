// Regional scalar transfer. Dependencies deliberately ignore source RGB/energy.
// Same finite paths, 128 lanes per atlas row; serial checkpoint RNG, parallel traces.
// 64 stateless scene-independent RNG checkpoints/row, precomputed once.
@compute @workgroup_size(64) fn response_seeds(@builtin(global_invocation_id) id:vec3<u32>){
 let i=id.x;if i>=response_count(){return;}var seed=hash(i+u.counts.w*91237u+519983u);
 for(var block=0u;block<64u;block++){
  filtered[i*16u+block/4u][block%4u]=bitcast<f32>(seed);
  if block<63u {for(var q=0u;q<64u;q++){seed=hash(seed+0x9e3779b9u);seed=hash(seed+0x9e3779b9u);}}
 }
}
var<workgroup> response_weights:array<vec4<f32>,384>;
var<workgroup> response_dependencies:array<Receiver,128>;
var<workgroup> response_row:u32;
fn response_region_texel(i:u32,lane:u32){
 let side=u.counts.x;let face=i/(side*side);let obj=face/6u;let k=face%6u;let pixel=i%(side*side);
 let p=face_point((vec2(f32(pixel%side),f32(pixel/side))+.5)/f32(side),boxes[obj],k);let n=axis(k);
 var cover=0u;for(var j=0u;j<u.dims.z;j++){if j!=obj && all(p>boxes[j].lo.xyz-vec3(.00001)) && all(p<boxes[j].hi.xyz+vec3(.00001)){cover|=1u<<j;}}
 let per_lane=u.counts.y*u.counts.y/128u;
 var hit_mask=0u;var shadow_mask=0u;var lo=vec3(1e20);var hi=vec3(-1e20);
 var weights:array<vec4<f32>,3>;for(var j=0u;j<3u;j++){weights[j]=vec4(0.);}
 if cover==0u {
  let start=lane*per_lane;let block=start/64u;
  var seed=bitcast<u32>(filtered[i*16u+block/4u][block%4u]);
  for(var q=0u;q<start%64u;q++){seed=hash(seed+0x9e3779b9u);seed=hash(seed+0x9e3779b9u);}
  let origin=p+n*.002;
  for(var q=0u;q<per_lane;q++){
   let j=lane*per_lane+q;
   let uv=(vec2(f32(j%u.counts.y),f32(j/u.counts.y))+pair(&seed))/f32(u.counts.y);
   let h=trace(origin,cosine(n,uv),1e20);if h.obj==0xffffffffu{continue;}hit_mask|=1u<<h.obj;
   let c=max(dot(h.n,u.sun.xyz),0.);if c==0.{continue;}
   lo=min(lo,h.p);hi=max(hi,h.p);var occluded=false;
   for(var b=0u;b<u.dims.z;b++){
    let interval=slab(h.p+h.n*.002,u.sun.xyz,boxes[b]);
    if interval.y>=max(interval.x,0.) && interval.y>.00001 {shadow_mask|=1u<<b;occluded=true;break;}
   }
   if !occluded{weights[h.obj/4u][h.obj%4u]+=c;}
  }
 }
 for(var j=0u;j<3u;j++){response_weights[lane*3u+j]=weights[j];}
 response_dependencies[lane].p=vec4(lo,bitcast<f32>(hit_mask));response_dependencies[lane].n=vec4(hi,bitcast<f32>(shadow_mask));
 workgroupBarrier();
 for(var step=64u;step>0u;step/=2u){
  if lane<step {
   for(var j=0u;j<3u;j++){response_weights[lane*3u+j]+=response_weights[(lane+step)*3u+j];}
   let a=response_dependencies[lane];let b=response_dependencies[lane+step];
   response_dependencies[lane].p=vec4(min(a.p.xyz,b.p.xyz),bitcast<f32>(bitcast<u32>(a.p.w)|bitcast<u32>(b.p.w)));
   response_dependencies[lane].n=vec4(max(a.n.xyz,b.n.xyz),bitcast<f32>(bitcast<u32>(a.n.w)|bitcast<u32>(b.n.w)));
  }
  workgroupBarrier();
 }
 if lane==0u {
  for(var j=0u;j<u.counts.z;j++){probes[i*u.counts.z+j]=response_weights[j]/f32(u.counts.y*u.counts.y);}
  result[i]=vec4(response_rgb(i),select(0.,1.,cover==0u));
  receivers[i].p=response_dependencies[0].p;receivers[i].n=response_dependencies[0].n;receivers[i].rho=vec4(bitcast<f32>(cover),0.,0.,0.);guides[i]=0u;
 }
}
@compute @workgroup_size(128) fn response_region_full(@builtin(workgroup_id) group:vec3<u32>,@builtin(local_invocation_index) lane:u32){
 let i=group.x+group.y*6144u;if i>=response_count(){return;}response_region_texel(i,lane);
}
@compute @workgroup_size(64) fn response_region_mark(@builtin(global_invocation_id) id:vec3<u32>){
 let i=id.x;if i>=response_count(){return;}var dirty=u.dims.w==3u;
 let change=u32(u.grid.z);
 if u.dims.w==2u {
  let side=u.counts.x;let face=i/(side*side);let obj=face/6u;let k=face%6u;
  let old=bitcast<u32>(receivers[i].p.w)|bitcast<u32>(receivers[i].n.w)|bitcast<u32>(receivers[i].rho.x)|(1u<<obj);
  dirty=(old&change)!=0u;
  if !dirty {
   let pixel=i%(side*side);let p=face_point((vec2(f32(pixel%side),f32(pixel/side))+.5)/f32(side),boxes[obj],k);let n=axis(k);
   for(var j=0u;j<u.dims.z;j++) {if (change&(1u<<j))!=0u {
    let b=boxes[j];let far=select(b.lo.xyz,b.hi.xyz,n>=vec3(0.));
    dirty=dirty || dot(far-p,n)>=-.001 || potential_shadow(i,b);
   }}
  }
 }
 guides[i]=select(0u,1u,dirty);
}
@compute @workgroup_size(128) fn response_region_repair(@builtin(workgroup_id) group:vec3<u32>,@builtin(local_invocation_index) lane:u32){
 let j=group.x+group.y*6144u;let k=u32(u.grid.x)+j;
 if lane==0u {response_row=0xffffffffu;if j<u32(u.grid.y) && k<atomicLoad(&work.counts[0]){response_row=work.ids[k];}}
 let i=workgroupUniformLoad(&response_row);if i==0xffffffffu{return;}response_region_texel(i,lane);
}
// Establish real storage hazards: independent marker kernels can overlap the work
// on Metal, producing a valid-looking but severely understated cross-pass span.
@compute @workgroup_size(1) fn response_boundary(){
 indirect[5]=u.dims.z ^ bitcast<u32>(result[0].x) ^ bitcast<u32>(probes[0].x) ^ bitcast<u32>(receivers[0].p.x) ^ guides[0] ^ atomicLoad(&work.counts[0]) ^ bitcast<u32>(boxes[0].lo.x);
}
