// Diagnostic per-primitive/per-face object-space atlas; no camera in its bake.
fn face_uv(p:vec3<f32>,b:Box,k:u32)->vec2<f32>{
 let v=clamp((p-b.lo.xyz)/(b.hi.xyz-b.lo.xyz),vec3(0.),vec3(1.));
 if k<2u{return v.yz;}if k<4u{return v.xz;}return v.xy;
}
fn face_point(uv:vec2<f32>,b:Box,k:u32)->vec3<f32>{
 var v=vec3(uv.x,uv.y,0.);if k<2u{v=vec3(0.,uv.x,uv.y);}else if k<4u{v=vec3(uv.x,0.,uv.y);}
 let a=k/2u;v[a]=select(0.,1.,(k&1u)==0u);return mix(b.lo.xyz,b.hi.xyz,v);
}
fn face_cell(b:Box,k:u32)->f32 {
 let s=b.hi.xyz-b.lo.xyz;var v=s.xy;if k<2u{v=s.yz;}else if k<4u{v=s.xz;}
 return max(v.x,v.y)/f32(u.counts.x);
}
@compute @workgroup_size(64) fn atlas_build(@builtin(global_invocation_id) id:vec3<u32>){
 let side=u.counts.x;let face_pixels=side*side;if id.x>=u.dims.z*6u*face_pixels{return;}
 let face=id.x/face_pixels;let obj=face/6u;let k=face%6u;let pixel=id.x%face_pixels;
 let uv=(vec2(f32(pixel%side),f32(pixel/side))+.5)/f32(side);let p=face_point(uv,boxes[obj],k);let n=axis(k);
 var valid=true;for(var j=0u;j<u.dims.z;j++){if j!=obj && all(p>boxes[j].lo.xyz-vec3(.00001)) && all(p<boxes[j].hi.xyz+vec3(.00001)){valid=false;}}
 probes[id.x]=vec4(0.);if !valid{return;}
 var seed=hash(id.x+u.counts.w*91237u+519983u);
 probes[id.x]=vec4(estimate(p+n*.002,n,u.counts.y,&seed),1.);
}
fn atlas_lookup(i:u32)->vec4<f32>{
 let obj=u32(receivers[i].p.w)-1u;let k=axis_index(receivers[i].n.xyz);let side=u.counts.x;
 let g=clamp(face_uv(receivers[i].p.xyz,boxes[obj],k)*f32(side)-.5,vec2(0.),vec2(f32(side-1u)));
 let cell=min(vec2<u32>(floor(g)),vec2(side-2u));let f=g-vec2<f32>(cell);var sum=vec3(0.);var sw=0.;
 for(var j=0u;j<4u;j++){
  let bits=vec2(j&1u,j>>1u);let v=probes[(obj*6u+k)*side*side+(cell.y+bits.y)*side+cell.x+bits.x];
  let w=select(1.-f.x,f.x,bits.x==1u)*select(1.-f.y,f.y,bits.y==1u);
  if v.w>0.{sum+=v.xyz*w;sw+=w;}
 }
 return vec4(sum/max(sw,.000001),sw);
}
fn atlas_risk(i:u32)->f32{
 let obj=u32(receivers[i].p.w)-1u;let p=receivers[i].p.xyz;let n=receivers[i].n.xyz;let b=boxes[obj];
 let edges=min(p-b.lo.xyz,b.hi.xyz-p);var d=1e20;
 for(var a=0u;a<3u;a++){if abs(n[a])<.5{d=min(d,edges[a]);}}
 for(var j=0u;j<u.dims.z;j++){if j!=obj{d=min(d,distance(p,clamp(p,boxes[j].lo.xyz,boxes[j].hi.xyz)));}}
 return d;
}
@compute @workgroup_size(8,8) fn atlas_query(@builtin(global_invocation_id) id:vec3<u32>){
 if id.x>=u.dims.x || id.y>=u.dims.y{return;}let i=id.y*u.dims.x+id.x;var value=vec4(0.);
 if receivers[i].p.w>0.{
  var direct=u.dims.w==25u;
  if u.dims.w==26u{
   let obj=u32(receivers[i].p.w)-1u;let k=axis_index(receivers[i].n.xyz);
   direct=atlas_risk(i)<=1.5*face_cell(boxes[obj],k);
  }
  var light=vec3(0.);var flag=0.;
  if !direct {let v=atlas_lookup(i);light=v.xyz*u.grid.xyz;direct=v.w<=.00001;}
  if direct{var seed=hash(i+u.counts.w*91237u+9923u);light=estimate(receivers[i].p.xyz+receivers[i].n.xyz*.002,receivers[i].n.xyz,8u,&seed);flag=2.;}
  value=vec4(light*receivers[i].rho.xyz,flag);
 }
 result[i]=value;let base=textureLoad(native,vec2<i32>(id.xy),0);
 textureStore(display,vec2<i32>(id.xy),vec4(srgb(base.xyz+value.xyz),base.w));
}
