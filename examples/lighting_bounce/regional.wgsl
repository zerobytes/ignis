// Dependency support for the FIXED finite bake samples; not continuous radiance certification.
fn region_count()->u32{return u.dims.z*6u*u.counts.x*u.counts.x;}
fn region_texel(i:u32){
 let side=u.counts.x;let face=i/(side*side);let obj=face/6u;let k=face%6u;let pixel=i%(side*side);
 let p=face_point((vec2(f32(pixel%side),f32(pixel/side))+.5)/f32(side),boxes[obj],k);let n=axis(k);
 var cover=0u;for(var j=0u;j<u.dims.z;j++){if j!=obj && all(p>boxes[j].lo.xyz-vec3(.00001)) && all(p<boxes[j].hi.xyz+vec3(.00001)){cover|=1u<<j;}}
 var hit_mask=0u;var shadow_mask=0u;var lo=vec3(1e20);var hi=vec3(-1e20);var sum=vec3(0.);
 if cover==0u {
  var seed=hash(i+u.counts.w*91237u+519983u);let origin=p+n*.002;
  for(var j=0u;j<u.counts.y*u.counts.y;j++){
   let uv=(vec2(f32(j%u.counts.y),f32(j/u.counts.y))+pair(&seed))/f32(u.counts.y);
   let h=trace(origin,cosine(n,uv),1e20);if h.obj==0xffffffffu{continue;}hit_mask|=1u<<h.obj;
   let c=max(dot(h.n,u.sun.xyz),0.);if c==0.{continue;}
   lo=min(lo,h.p);hi=max(hi,h.p);var occluded=false;
   for(var b=0u;b<u.dims.z;b++){
    let interval=slab(h.p+h.n*.002,u.sun.xyz,boxes[b]);
    if interval.y>=max(interval.x,0.) && interval.y>.00001 {shadow_mask|=1u<<b;occluded=true;break;}
   }
   if !occluded{sum+=boxes[h.obj].color.xyz*u.energy.xyz*c;}
  }
 }
 probes[i]=vec4(sum/f32(u.counts.y*u.counts.y),select(0.,1.,cover==0u));
 receivers[i].p=vec4(lo,bitcast<f32>(hit_mask));receivers[i].n=vec4(hi,bitcast<f32>(shadow_mask));
 receivers[i].rho=vec4(bitcast<f32>(cover),0.,0.,0.);
}
@compute @workgroup_size(64) fn region_full(@builtin(global_invocation_id) id:vec3<u32>){if id.x<region_count(){region_texel(id.x);guides[id.x]=0u;}}
fn projection(lo:vec3<f32>,hi:vec3<f32>,a:vec3<f32>)->vec2<f32>{
 let center=(lo+hi)*.5;let radius=dot((hi-lo)*.5,abs(a));return vec2(dot(center,a)-radius,dot(center,a)+radius);
}
fn potential_shadow(i:u32,b:Box)->bool{
 let lo=receivers[i].p.xyz;let hi=receivers[i].n.xyz;if any(lo>hi){return false;}
 // Shadow rays start at endpoint+n*.002. Expand unknown endpoint-normal offset.
 let l=lo-vec3(.0021);let h=hi+vec3(.0021);
 let t=normalize(cross(select(vec3(0.,1.,0.),vec3(1.,0.,0.),abs(u.sun.y)>.9),u.sun.xyz));let v=cross(u.sun.xyz,t);
 let a=projection(l,h,t);let ba=projection(b.lo.xyz,b.hi.xyz,t);if a.y<ba.x-.001 || ba.y<a.x-.001{return false;}
 let c=projection(l,h,v);let bc=projection(b.lo.xyz,b.hi.xyz,v);if c.y<bc.x-.001 || bc.y<c.x-.001{return false;}
 let z=projection(l,h,u.sun.xyz);let bz=projection(b.lo.xyz,b.hi.xyz,u.sun.xyz);return z.x<=bz.y+.001;
}
@compute @workgroup_size(64) fn region_mark(@builtin(global_invocation_id) id:vec3<u32>){
 let i=id.x;if i>=region_count(){return;}let kind=u.dims.w;var dirty=false;
 if kind==3u{dirty=true;}
 else if kind!=0u {
  let change=u.counts.z;let bit=1u<<change;let hit=bitcast<u32>(receivers[i].p.w);
  dirty=(hit&bit)!=0u;
  if kind==2u {
   let face=i/(u.counts.x*u.counts.x);let obj=face/6u;let k=face%6u;
   dirty=dirty || obj==change || ((bitcast<u32>(receivers[i].n.w)|bitcast<u32>(receivers[i].rho.x))&bit)!=0u;
   if !dirty {
    let pixel=i%(u.counts.x*u.counts.x);let p=face_point((vec2(f32(pixel%u.counts.x),f32(pixel/u.counts.x))+.5)/f32(u.counts.x),boxes[obj],k);
    let n=axis(k);let b=boxes[change];let far=select(b.lo.xyz,b.hi.xyz,n>=vec3(0.));
    // A new primary intersection must lie in the outward hemisphere; includes burial.
    dirty=dot(far-p,n)>=-.001 || potential_shadow(i,b);
   }
  }
 }
 guides[i]=select(0u,1u,dirty);
}
@compute @workgroup_size(64) fn region_compact(@builtin(global_invocation_id) id:vec3<u32>,@builtin(local_invocation_index) lane:u32){
 var dirty=0u;if id.x<region_count(){dirty=guides[id.x];}low_scan[lane]=dirty;workgroupBarrier();
 for(var stride=1u;stride<64u;stride*=2u){var a=0u;if lane>=stride{a=low_scan[lane-stride];}workgroupBarrier();low_scan[lane]+=a;workgroupBarrier();}
 if lane==0u{low_base=atomicAdd(&work.counts[0],low_scan[63]);}workgroupBarrier();
 if dirty!=0u{work.ids[low_base+low_scan[lane]-1u]=id.x;}
}
@compute @workgroup_size(64) fn region_repair(@builtin(global_invocation_id) id:vec3<u32>){
 if id.x>=atomicLoad(&work.counts[0]){return;}let i=work.ids[id.x];region_texel(i);guides[i]=0u;
}
fn region_pending(i:u32)->bool{
 let obj=u32(receivers[i].p.w)-1u;let k=axis_index(receivers[i].n.xyz);let side=u.counts.x;
 let g=clamp(face_uv(receivers[i].p.xyz,boxes[obj],k)*f32(side)-.5,vec2(0.),vec2(f32(side-1u)));
 let cell=min(vec2<u32>(floor(g)),vec2(side-2u));
 for(var y=0u;y<2u;y++){for(var x=0u;x<2u;x++){if guides[(obj*6u+k)*side*side+(cell.y+y)*side+cell.x+x]!=0u{return true;}}}return false;
}
@compute @workgroup_size(8,8) fn region_query(@builtin(global_invocation_id) id:vec3<u32>){
 if id.x>=u.dims.x || id.y>=u.dims.y{return;}let i=id.y*u.dims.x+id.x;var value=vec4(0.);
 if receivers[i].p.w>0.{
  var direct=region_pending(i);var light=vec3(0.);
  if !direct{let a=atlas_lookup(i);light=a.xyz;direct=a.w<=.00001;}
  if direct{var seed=hash(i+u.counts.w*91237u+9923u);light=estimate(receivers[i].p.xyz+receivers[i].n.xyz*.002,receivers[i].n.xyz,8u,&seed);}
  value=vec4(receivers[i].rho.xyz*light,select(0.,2.,direct));
 }
 result[i]=value;let base=textureLoad(native,vec2<i32>(id.xy),0);textureStore(display,vec2<i32>(id.xy),vec4(srgb(base.xyz+value.xyz),base.w));
}
