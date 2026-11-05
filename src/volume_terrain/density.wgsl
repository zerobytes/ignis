// GPU Gems 3 ch.1 density operations; native Ignis seeded noise, not texture lookups.
@group(0) @binding(0) var<storage,read> p:array<vec4<f32>>;
@group(0) @binding(1) var<storage,read_write> density:array<f32>;
fn hash32(x:u32)->u32 {var v=x;v=v^(v>>16u);v=v*0x7feb352du;v=v^(v>>15u);v=v*0x846ca68bu;return v^(v>>16u);}
fn rol13(v:u32)->u32{return (v<<13u)|(v>>19u);}
fn noise(q:vec3<f32>,seed:u32)->f32 {
 let cell=vec3<i32>(floor(q));let f0=fract(q);let f=f0*f0*f0*(f0*(f0*6.-15.)+10.);var result=0.;
 for(var z=0u;z<2u;z++){for(var y=0u;y<2u;y++){for(var x=0u;x<2u;x++){
  let c=bitcast<vec3<u32>>(cell)+vec3<u32>(x,y,z);
  let h=hash32(c.x^hash32(c.y)^rol13(hash32(c.z))^seed);
  let v=f32(h)*(2./4294967295.)-1.;
  result+=v*select(1.-f.x,f.x,x==1u)*select(1.-f.y,f.y,y==1u)*select(1.-f.z,f.z,z==1u);
 }}}return result;
}
fn altitude(q:vec3<f32>)->f32 {if(p[3].x==0.){return q.y-p[3].y;}return length(q-p[2].xyz)-p[2].w;}
fn snap(t:f32,m:f32)->f32 {if(t<=0.5){return 0.5*pow(2.*t,m);}return 1.-0.5*pow(2.*(1.-t),m);}
fn field(original:vec3<f32>)->f32 {
 let original_h=altitude(original);var q=original;let noise_p=original*p[9].xyz;let seed=bitcast<u32>(p[1].w);
 if(p[4].z!=0.){for(var axis=0u;axis<3u;axis++){
  let s=seed+0x85ebca6bu*(axis+1u);
  q[axis]+=p[4].y*p[10][axis]*(noise(noise_p*p[4].x,s)*0.64+noise(noise_p*(p[4].x*0.461),s^0xc2b2ae35u)*0.32);
 }}
 let h=altitude(q);var d=-h*p[3].z;
 if(p[8].w!=0.){d+=clamp((p[8].x-original_h)/p[8].y,0.,1.)*p[8].z;}
 if(p[7].w!=0.){let weight=clamp(p[7].y-abs(h-p[7].x),0.,1.)*p[7].w;d+=(p[7].z-d)*weight;}
 if(p[6].w!=0.){let height=mix(original_h,h,p[6].y);let fy=-height/p[5].x+p[6].z;let fraction=fract(fy);let t=snap(fraction,p[6].x);d+=(floor(fy)+t-p[6].z)*p[5].y+(t-fraction)*p[5].z;}
 for(var i=0u;i<u32(p[3].w);i++){
  let k=11u+i*4u;let o=p[k];let rotation=mat3x3<f32>(p[k+1u].xyz,p[k+2u].xyz,p[k+3u].xyz);
  d+=noise((rotation*(q*p[9].xyz))*o.x,bitcast<u32>(o.z))*o.y;
 }return d;
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>){
 let dims=vec3<u32>(p[1].xyz);let count=dims.x*dims.y*dims.z;let i=id.x;if(i>=count){return;}
 let cell=vec3<u32>(i%dims.x,(i/dims.x)%dims.y,i/(dims.x*dims.y));
 density[i]=field(p[0].xyz+vec3<f32>(cell)*vec3<f32>(p[0].w,p[5].w,p[0].w));
}
