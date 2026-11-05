// One-bounce scalar material response, fixed geometry/sun direction. Max12slots.
fn response_count()->u32{return u.dims.z*6u*u.counts.x*u.counts.x;}
fn response_rgb(i:u32)->vec3<f32>{
 var sum=vec3(0.);for(var j=0u;j<u.dims.z;j++){sum+=probes[i*u.counts.z+j/4u][j%4u]*boxes[j].color.xyz;}
 return sum*u.energy.xyz;
}
@compute @workgroup_size(64) fn response_build(@builtin(global_invocation_id) id:vec3<u32>){
 let i=id.x;if i>=response_count(){return;}let side=u.counts.x;let face=i/(side*side);let obj=face/6u;let k=face%6u;let pixel=i%(side*side);
 let p=face_point((vec2(f32(pixel%side),f32(pixel/side))+.5)/f32(side),boxes[obj],k);let n=axis(k);
 var valid=true;for(var j=0u;j<u.dims.z;j++){if j!=obj && all(p>boxes[j].lo.xyz-vec3(.00001)) && all(p<boxes[j].hi.xyz+vec3(.00001)){valid=false;}}
 var weights:array<vec4<f32>,3>;for(var j=0u;j<3u;j++){weights[j]=vec4(0.);}
 if valid {
  var seed=hash(i+u.counts.w*91237u+519983u);let origin=p+n*.002;
  for(var j=0u;j<u.counts.y*u.counts.y;j++){
   let uv=(vec2(f32(j%u.counts.y),f32(j/u.counts.y))+pair(&seed))/f32(u.counts.y);
   let h=trace(origin,cosine(n,uv),1e20);if h.obj==0xffffffffu{continue;}
   let c=max(dot(h.n,u.sun.xyz),0.);if c==0. || blocked(h.p+h.n*.002,u.sun.xyz,1e20){continue;}
   weights[h.obj/4u][h.obj%4u]+=c;
  }
 }
 var sum=vec3(0.);
 for(var j=0u;j<u.counts.z;j++){probes[i*u.counts.z+j]=weights[j]/f32(u.counts.y*u.counts.y);}
 for(var j=0u;j<u.dims.z;j++){sum+=weights[j/4u][j%4u]/f32(u.counts.y*u.counts.y)*boxes[j].color.xyz;}
 result[i]=vec4(sum*u.energy.xyz,select(0.,1.,valid));
}
@compute @workgroup_size(64) fn response_combine(@builtin(global_invocation_id) id:vec3<u32>){
 let i=id.x;if i>=response_count(){return;}result[i]=vec4(response_rgb(i),result[i].w);
}
