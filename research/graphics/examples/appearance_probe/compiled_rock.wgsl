struct U {
 inverse:mat4x4<f32>, eye:vec4<f32>, center_extent:vec4<f32>,
 right0:vec4<f32>, up0:vec4<f32>, view0:vec4<f32>,
 right1:vec4<f32>, up1:vec4<f32>, view1:vec4<f32>,
 rect:vec4<f32>, grid:vec4<u32>, indices:vec4<u32>, weights_screen:vec4<f32>
};
@group(0) @binding(0) var<uniform> g:U;
@group(0) @binding(1) var<storage,read> directory:array<u32>;
@group(0) @binding(2) var<storage,read> maps:array<u32>;
@group(0) @binding(3) var<storage,read> responses:array<u32>;
@vertex fn vertex(@builtin(vertex_index) i:u32)->@builtin(position)vec4<f32>{
 let corners=array<vec2<f32>,6>(vec2(0.,0.),vec2(1.,0.),vec2(0.,1.),vec2(0.,1.),vec2(1.,0.),vec2(1.,1.));
 let uv=mix(g.rect.xy,g.rect.zw,corners[i]);return vec4(uv*vec2(2.,-2.)+vec2(-1.,1.),0.,1.);
}
fn code(p:vec2<i32>,v:u32)->u32 {
 if any(p<vec2(0)) || any(p>=vec2<i32>(i32(g.grid.x))){return 0u;}
 let q=vec2<u32>(p);let tiles=g.grid.y;
 let offset=directory[v*tiles*tiles+(q.y/8u)*tiles+q.x/8u];
 if offset==0xffffffffu{return 0u;}
 return maps[offset+(q.y%8u)*8u+q.x%8u];
}
fn response(w:u32,v:u32)->vec4<f32> {
 let id=w&65535u;if id==0u{return vec4(0.);}
 let a=responses[(v*g.grid.z+g.indices.z)*g.grid.w+id-1u];
 let b=responses[(v*g.grid.z+g.indices.w)*g.grid.w+id-1u];
 let ca=vec4<f32>(f32(a&1023u),f32((a>>10u)&1023u),f32((a>>20u)&1023u),f32(a>>30u));
 let cb=vec4<f32>(f32(b&1023u),f32((b>>10u)&1023u),f32((b>>20u)&1023u),f32(b>>30u));
 return mix(ca,cb,g.weights_screen.y)*vec4(2./1023.,2./1023.,2./1023.,1./3.);
}
fn surface(ray:vec3<f32>,right:vec3<f32>,up:vec3<f32>,view:vec3<f32>,v:u32)->vec4<f32>{
 let origin=g.eye.xyz-g.center_extent.xyz;let denom=dot(ray,view);
 if abs(denom)<0.00001{return vec4(0.);}
 var h=g.center_extent.w;var uv=vec2(0.);
 for(var i=0;i<2;i++) {
  let t=(h-dot(origin,view))/denom;let p=origin+ray*t;
  uv=vec2(dot(p,right),-dot(p,up))/(2.*g.center_extent.w)+vec2(0.5);
  let w=code(vec2<i32>(floor(uv*f32(g.grid.x))),v);
  if (w&65535u)==0u{return vec4(0.);}
  h=(f32(w>>16u)/65535.*2.-1.)*g.center_extent.w;
 }
 let t=(h-dot(origin,view))/denom;let p=origin+ray*t;
 uv=vec2(dot(p,right),-dot(p,up))/(2.*g.center_extent.w)+vec2(0.5);
 let xy=uv*f32(g.grid.x)-vec2(0.5);let base=vec2<i32>(floor(xy));let f=fract(xy);
 let a=mix(response(code(base,v),v),response(code(base+vec2(1,0),v),v),f.x);
 let b=mix(response(code(base+vec2(0,1),v),v),response(code(base+vec2(1,1),v),v),f.x);
 return mix(a,b,f.y);
}
// Diagnostic minimum lookup: one view, one light, one depth correction; no spatial/angular filter.
fn direct_surface(ray:vec3<f32>)->vec4<f32> {
 let origin=g.eye.xyz-g.center_extent.xyz;let view=g.view0.xyz;let denom=dot(ray,view);
 if abs(denom)<0.00001{return vec4(0.);}
 var h=g.center_extent.w;var w=0u;
 for(var i=0;i<2;i++) {
  let p=origin+ray*((h-dot(origin,view))/denom);
  let uv=vec2(dot(p,g.right0.xyz),-dot(p,g.up0.xyz))/(2.*g.center_extent.w)+vec2(0.5);
  w=code(vec2<i32>(floor(uv*f32(g.grid.x))),g.indices.x);
  if (w&65535u)==0u{return vec4(0.);}
  h=(f32(w>>16u)/65535.*2.-1.)*g.center_extent.w;
 }
 let a=responses[(g.indices.x*g.grid.z+g.indices.z)*g.grid.w+(w&65535u)-1u];
 return vec4(f32(a&1023u)*2./1023.,f32((a>>10u)&1023u)*2./1023.,f32((a>>20u)&1023u)*2./1023.,f32(a>>30u)/3.);
}
@fragment fn fragment(@builtin(position) pixel:vec4<f32>)->@location(0)vec4<f32>{
 let ndc=pixel.xy/g.weights_screen.zw*vec2(2.,-2.)+vec2(-1.,1.);
 let w=g.inverse*vec4(ndc,0.1,1.);let ray=w.xyz/w.w-g.eye.xyz;
 if g.right0.w>0. {let c=direct_surface(ray);return vec4(c.rgb+vec3(0.04,0.07,0.11)*(1.-c.a),c.a);}
 let a=surface(ray,g.right0.xyz,g.up0.xyz,g.view0.xyz,g.indices.x);
 let b=surface(ray,g.right1.xyz,g.up1.xyz,g.view1.xyz,g.indices.y);
 let c=mix(a,b,g.weights_screen.x);
 return vec4(c.rgb+vec3(0.04,0.07,0.11)*(1.-c.a),c.a);
}
