struct Globals { matrix:mat4x4<f32>, inverse:mat4x4<f32>, eye:vec4<f32>, light:vec4<f32>, screen:vec4<f32> };
struct Region { center:vec4<f32>, extent:vec4<f32>, info:vec4<u32> };
struct Instance { position:vec4<f32>, info:vec4<u32> };
@group(0) @binding(0) var<uniform> g:Globals;
@group(0) @binding(1) var<storage,read> regions:array<Region>;
@group(0) @binding(2) var<storage,read> instances:array<Instance>;
@group(0) @binding(3) var colors:texture_2d<f32>;
@group(0) @binding(4) var normals:texture_2d<f32>;
struct Surface { @builtin(position) clip:vec4<f32>, @location(0) world:vec3<f32>, @location(1) normal:vec3<f32>, @location(2) color:vec3<f32>, @location(3) @interpolate(flat) material:u32, @location(4) @interpolate(flat) instance:u32 };
@vertex fn geometry(@location(0) p:vec3<f32>,@location(1)n:vec3<f32>,@location(2)c:vec3<f32>,@location(3)m:u32,@builtin(instance_index) id:u32)->Surface {
 let o=instances[id];let world=p*o.position.w+o.position.xyz;var s:Surface;s.clip=g.matrix*vec4<f32>(world,1.0);s.world=world;s.normal=n;s.color=c;s.material=m;s.instance=id;return s;
}
fn shade(color:vec3<f32>,normal:vec3<f32>,world:vec3<f32>,material:u32)->vec3<f32>{
 let roughness=array<f32,7>(0.88,0.66,0.28,0.18,0.24,0.92,0.57)[material];
 let infollic=select(0.0,1.0,material==2u||material==3u);let n=normalize(normal);let v=normalize(g.eye.xyz-world);let l=g.light.xyz;let h=normalize(l+v);
 let nv=max(dot(n,v),0.0001);let nl=max(dot(n,l),0.0);let nh=max(dot(n,h),0.0);let a2=pow(roughness,4.0);let denom=nh*nh*(a2-1.0)+1.0;let d=a2/(3.14159265*denom*denom);let k=(roughness+1.0)*(roughness+1.0)/8.0;let visibility=(nv/(nv*(1.0-k)+k))*(nl/(nl*(1.0-k)+k));let f0=mix(vec3<f32>(0.04),color,infollic);let f=f0+(vec3<f32>(1.0)-f0)*pow(1.0-max(dot(h,v),0.0),5.0);let spec=d*visibility*f/max(4.0*nv*nl,0.0001);let diffuse=(vec3<f32>(1.0)-f)*(1.0-infollic)*color/3.14159265;
 return color*0.08+(diffuse+spec)*nl*3.14159265;
}
@fragment fn surface(s:Surface,@builtin(front_facing)front:bool)->@location(0)vec4<f32>{
 let id=instances[s.instance].info.z;
 if g.screen.z>1.5 {return vec4<f32>(shade(s.color,s.normal*select(-1.0,1.0,front),s.world,s.material),f32(instances[s.instance].info.w));}
 if g.screen.z>0.5 {return vec4<f32>(select(vec3<f32>(0.04,0.3,0.95),vec3<f32>(0.12),id==0u),f32(id));}
 return vec4<f32>(shade(s.color,s.normal*select(-1.0,1.0,front),s.world,s.material),f32(id));
}
struct Quad { @builtin(position) position:vec4<f32>, @location(0) @interpolate(flat) instance:u32 };
@vertex fn quad(@builtin(vertex_index) vertex:u32,@builtin(instance_index) id:u32)->Quad{
 let o=instances[id];let r=regions[o.info.x];var lo=vec2<f32>(1.0);var hi=vec2<f32>(-1.0);
 // Include filter support around the local cache; no per-triangle proxy construction.
 for(var i=0u;i<8u;i++){let sign=vec3<f32>(select(-1.0,1.0,(i&1u)!=0u),select(-1.0,1.0,(i&2u)!=0u),select(-1.0,1.0,(i&4u)!=0u));let p=(r.center.xyz+sign*r.center.w)*o.position.w+o.position.xyz;let q=g.matrix*vec4<f32>(p,1.0);lo=min(lo,q.xy/q.w);hi=max(hi,q.xy/q.w);}
 lo=max(lo,vec2<f32>(-1.0));hi=min(hi,vec2<f32>(1.0));
 let corners=array<vec2<f32>,6>(vec2<f32>(0,0),vec2<f32>(1,0),vec2<f32>(0,1),vec2<f32>(0,1),vec2<f32>(1,0),vec2<f32>(1,1));
 var q:Quad;q.position=vec4<f32>(mix(lo,hi,corners[vertex]),0.0,1.0);q.instance=id;return q;
}
fn basis(face:u32)->mat3x3<f32>{
 let u=array<vec3<f32>,6>(vec3<f32>(0,0,1),vec3<f32>(0,0,-1),vec3<f32>(1,0,0),vec3<f32>(1,0,0),vec3<f32>(-1,0,0),vec3<f32>(1,0,0));
 let v=array<vec3<f32>,6>(vec3<f32>(0,1,0),vec3<f32>(0,1,0),vec3<f32>(0,0,1),vec3<f32>(0,0,-1),vec3<f32>(0,1,0),vec3<f32>(0,1,0));
 let w=array<vec3<f32>,6>(vec3<f32>(1,0,0),vec3<f32>(-1,0,0),vec3<f32>(0,1,0),vec3<f32>(0,-1,0),vec3<f32>(0,0,1),vec3<f32>(0,0,-1));return mat3x3<f32>(u[face],v[face],w[face]);
}
struct Cache { color:vec4<f32>, normal:vec4<f32> };
fn load_texel(index:u32)->Cache {let width=textureDimensions(colors).x;let xy=vec2<i32>(i32(index%width),i32(index/width));return Cache(textureLoad(colors,xy,0),textureLoad(normals,xy,0));}
fn lookup(uv:vec2<f32>,base:u32,lod:u32)->Cache {
 if any(uv<vec2<f32>(0))||any(uv>vec2<f32>(1)){return Cache(vec4<f32>(0),vec4<f32>(0));}
 let size=16u>>lod;let offsets=array<u32,5>(0u,256u,320u,336u,340u);let p=clamp(uv*f32(size)-0.5,vec2<f32>(0),vec2<f32>(f32(size-1u)));let a=vec2<u32>(floor(p));let b=min(a+vec2<u32>(1u),vec2<u32>(size-1u));let f=fract(p);let start=base+offsets[lod];
 let aa=load_texel(start+a.y*size+a.x);let ba=load_texel(start+a.y*size+b.x);let ab=load_texel(start+b.y*size+a.x);let bb=load_texel(start+b.y*size+b.x);
 return Cache(mix(mix(aa.color,ba.color,f.x),mix(ab.color,bb.color,f.x),f.y),mix(mix(aa.normal,ba.normal,f.x),mix(ab.normal,bb.normal,f.x),f.y));
}
struct Result { @location(0) color:vec4<f32>, @builtin(frag_depth) depth:f32 };
@fragment fn summary(q:Quad)->Result{
 let o=instances[q.instance];let r=regions[o.info.x];let axes=basis(o.info.y);let ndc=q.position.xy/g.screen.xy*vec2<f32>(2,-2)+vec2<f32>(-1,1);let h=g.inverse*vec4<f32>(ndc,0.5,1.0);let ray=normalize(h.xyz/h.w-g.eye.xyz);let origin=(g.eye.xyz-o.position.xyz)/o.position.w-r.center.xyz;
 let denom=dot(ray,axes[2]);if abs(denom)<0.01{discard;}
 var t=-dot(origin,axes[2])/denom;var p=origin+ray*t;var uv=vec2<f32>(dot(p,axes[0]),dot(p,axes[1]))/(2.0*r.center.w)+0.5;let base=r.info.x+o.info.y*341u;
 for(var k=0u;k<4u;k++){let a=lookup(uv,base,0u);if a.color.a<0.01{break;}t=(a.normal.w/a.color.a-dot(origin,axes[2]))/denom;p=origin+ray*t;uv=vec2<f32>(dot(p,axes[0]),dot(p,axes[1]))/(2.0*r.center.w)+0.5;}
 let c=lookup(uv,base,o.info.w);if c.color.a<0.5||t<0.0||length(c.normal.xyz)<0.0001{discard;}
 let world=(p+r.center.xyz)*o.position.w+o.position.xyz;let clip=g.matrix*vec4<f32>(world,1.0);let depth=clip.z/clip.w;if depth<0.0||depth>1.0{discard;}
 var result:Result;result.depth=depth;
 let rgb=shade(c.color.rgb/c.color.a,normalize(c.normal.xyz),world,r.info.y);result.color=vec4<f32>(select(rgb,vec3<f32>(1.0,0.42,0.025),g.screen.z>0.5),f32(o.info.z));return result;
}
