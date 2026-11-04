// Sun-lit diffuse endpoint, one indirect bounce, unbounded box visibility.
struct Params {
 inv:mat4x4<f32>, eye:vec4<f32>, dims:vec4<u32>,
 sun:vec4<f32>, energy:vec4<f32>, grid:vec4<f32>, counts:vec4<u32>,
}
struct Box { lo:vec4<f32>, hi:vec4<f32>, color:vec4<f32> }
struct Receiver { p:vec4<f32>, n:vec4<f32>, rho:vec4<f32> }
struct Hit { t:f32, obj:u32, p:vec3<f32>, n:vec3<f32> }
@group(0) @binding(0) var<uniform> u:Params;
@group(0) @binding(1) var<storage,read> boxes:array<Box>;
@group(0) @binding(2) var<storage,read_write> receivers:array<Receiver>;
@group(0) @binding(3) var native:texture_2d<f32>;
@group(0) @binding(4) var display:texture_storage_2d<rgba8unorm,write>;
@group(0) @binding(5) var<storage,read_write> result:array<vec4<f32>>;
@group(0) @binding(6) var<storage,read_write> probes:array<vec4<f32>>;
fn hash(x:u32)->u32 {
 var v=x;v=(v^(v>>16u))*0x7feb352du;v=(v^(v>>15u))*0x846ca68bu;return v^(v>>16u);
}
fn random(seed:ptr<function,u32>)->f32 {
 *seed=hash(*seed+0x9e3779b9u);return f32(*seed>>8u)/16777216.;
}
fn pair(seed:ptr<function,u32>)->vec2<f32> {
 let a=random(seed);let b=random(seed);return vec2(a,b);
}
fn slab(p:vec3<f32>,d:vec3<f32>,b:Box)->vec2<f32> {
 let a=(b.lo.xyz-p)/d;let c=(b.hi.xyz-p)/d;let lo=min(a,c);let hi=max(a,c);
 return vec2(max(max(lo.x,lo.y),lo.z),min(min(hi.x,hi.y),hi.z));
}
fn normal(p:vec3<f32>,b:Box)->vec3<f32> {
 let v=(p-(b.lo.xyz+b.hi.xyz)*.5)/((b.hi.xyz-b.lo.xyz)*.5);
 if abs(v.x)>abs(v.y) && abs(v.x)>abs(v.z){return vec3(sign(v.x),0.,0.);}
 if abs(v.y)>abs(v.z){return vec3(0.,sign(v.y),0.);}
 return vec3(0.,0.,sign(v.z));
}
fn trace(p:vec3<f32>,d:vec3<f32>,limit:f32)->Hit {
 var h:Hit;h.t=limit;h.obj=0xffffffffu;h.p=vec3(0.);h.n=vec3(0.);
 for(var j=0u;j<u.dims.z;j++) {
  let interval=slab(p,d,boxes[j]);
  if interval.x>.00001 && interval.y>=interval.x && interval.x<h.t {
   h.t=interval.x;h.obj=j;
  }
 }
 if h.obj!=0xffffffffu {h.p=p+h.t*d;h.n=normal(h.p,boxes[h.obj]);}
 return h;
}
fn blocked(p:vec3<f32>,d:vec3<f32>,limit:f32)->bool {
 for(var j=0u;j<u.dims.z;j++) {
  let h=slab(p,d,boxes[j]);
  if h.y>=max(h.x,0.) && h.y>.00001 && h.x<limit {return true;}
 }
 return false;
}
fn cosine(n:vec3<f32>,v:vec2<f32>)->vec3<f32> {
 let t=normalize(cross(select(vec3(0.,1.,0.),vec3(1.,0.,0.),abs(n.y)>.9),n));
 return t*(sqrt(v.x)*cos(6.283185307*v.y))+cross(n,t)*(sqrt(v.x)*sin(6.283185307*v.y))+n*sqrt(1.-v.x);
}
fn endpoint(p:vec3<f32>,d:vec3<f32>)->vec3<f32> {
 let h=trace(p,d,1e20);if h.obj==0xffffffffu{return vec3(0.);}
 let c=max(dot(h.n,u.sun.xyz),0.);
 if c==0. || blocked(h.p+h.n*.002,u.sun.xyz,1e20){return vec3(0.);}
 return boxes[h.obj].color.xyz*u.energy.xyz*c;
}
fn estimate(p:vec3<f32>,n:vec3<f32>,side:u32,seed:ptr<function,u32>)->vec3<f32> {
 var sum=vec3(0.);
 for(var j=0u;j<side*side;j++) {
  let v=(vec2(f32(j%side),f32(j/side))+pair(seed))/f32(side);
  sum+=endpoint(p,cosine(n,v));
 }
 return sum/f32(side*side);
}
fn axis(k:u32)->vec3<f32> {
 switch k {
  case 0u:{return vec3(1.,0.,0.);}case 1u:{return vec3(-1.,0.,0.);}
  case 2u:{return vec3(0.,1.,0.);}case 3u:{return vec3(0.,-1.,0.);}
  case 4u:{return vec3(0.,0.,1.);}default:{return vec3(0.,0.,-1.);}
 }
}
fn axis_index(n:vec3<f32>)->u32 {
 if n.x>.5{return 0u;}if n.x<-.5{return 1u;}
 if n.y>.5{return 2u;}if n.y<-.5{return 3u;}if n.z>.5{return 4u;}return 5u;
}
fn coords(i:u32)->vec3<u32> {
 return vec3(i%u.counts.x,(i/u.counts.x)%u.counts.y,i/(u.counts.x*u.counts.y));
}
fn index(v:vec3<u32>)->u32 {return v.x+u.counts.x*(v.y+u.counts.y*v.z);}
fn probe_position(i:u32)->vec3<f32>{return u.grid.xyz+vec3<f32>(coords(i))*u.grid.w;}
@compute @workgroup_size(8,8) fn primary(@builtin(global_invocation_id) id:vec3<u32>) {
 if id.x>=u.dims.x || id.y>=u.dims.y{return;}let i=id.y*u.dims.x+id.x;
 let xy=vec2((f32(id.x)+.5)/f32(u.dims.x)*2.-1.,1.-(f32(id.y)+.5)/f32(u.dims.y)*2.);
 let q=u.inv*vec4(xy,0.,1.);let d=normalize(q.xyz/q.w-u.eye.xyz);
 let h=trace(u.eye.xyz,d,1e20);receivers[i].p=vec4(0.);
 if h.obj==0xffffffffu{return;}
 receivers[i].p=vec4(h.p,f32(h.obj+1u));receivers[i].n=vec4(h.n,0.);receivers[i].rho=boxes[h.obj].color;
}
@compute @workgroup_size(64) fn build(@builtin(global_invocation_id) id:vec3<u32>) {
 let total=u.counts.x*u.counts.y*u.counts.z;
 if id.x>=total*6u{return;}let i=id.x/6u;let k=id.x%6u;let p=probe_position(i);
 var valid=true;
 for(var j=0u;j<u.dims.z;j++) {
  if all(p>boxes[j].lo.xyz-vec3(.0001)) && all(p<boxes[j].hi.xyz+vec3(.0001)){valid=false;}
 }
 probes[id.x]=vec4(0.);if !valid{return;}
 var seed=hash(i*737u+k*223u+u.counts.w*91237u+113u);
 probes[id.x]=vec4(estimate(p,axis(k),16u,&seed),1.);
}
fn lookup(i:u32,gated:bool,seed:ptr<function,u32>)->vec4<f32> {
 let p=receivers[i].p.xyz;let n=receivers[i].n.xyz;let origin=p+n*.002;
 let g=clamp((origin-u.grid.xyz)/u.grid.w,vec3(0.),vec3<f32>(u.counts.xyz-vec3(1u)));
 let cell=min(vec3<u32>(floor(g)),u.counts.xyz-vec3(2u));let f=g-vec3<f32>(cell);
 let k=axis_index(n);var sum=vec3(0.);var weight=0.;
 for(var j=0u;j<8u;j++) {
  let bits=vec3(j&1u,(j>>1u)&1u,(j>>2u)&1u);let node=index(cell+bits);
  let value=probes[node*6u+k];if value.w==0.{continue;}
  let factors=select(vec3(1.)-f,f,bits==vec3(1u));let w=factors.x*factors.y*factors.z;
  if w<=0.{continue;}
  if gated {let delta=probe_position(node)-origin;let distance=length(delta);
   if distance>.0001 && blocked(origin,delta/distance,max(0.,distance-.001)){continue;}
  }
  sum+=w*value.xyz;weight+=w;
 }
 if weight>.00001{return vec4(sum/weight,0.);}
 return vec4(estimate(origin,n,4u,seed),1.);
}
fn srgb(c:vec3<f32>)->vec3<f32>{let x=max(c,vec3(0.));return select(1.055*pow(x,vec3(1./2.4))-.055,12.92*x,x<=vec3(.0031308));}
@compute @workgroup_size(8,8) fn shade(@builtin(global_invocation_id) id:vec3<u32>) {
 if id.x>=u.dims.x || id.y>=u.dims.y{return;}let i=id.y*u.dims.x+id.x;
 let mode=u.dims.w;var value=vec4(0.);
 if receivers[i].p.w>0. && mode!=0u {
  var seed=hash(i+u.counts.w*91237u+select(9923u,181331u,mode>=7u));
  if mode>=3u && mode<=6u {value=lookup(i,mode==4u || mode==6u,&seed);}
  else {let side=select(select(4u,8u,mode==2u),64u,mode>=7u);
   value=vec4(estimate(receivers[i].p.xyz+receivers[i].n.xyz*.002,receivers[i].n.xyz,side,&seed),0.);
  }
  value=vec4(receivers[i].rho.xyz*value.xyz,value.w);
 }
 result[i]=value;let base=textureLoad(native,vec2<i32>(id.xy),0);
 textureStore(display,vec2<i32>(id.xy),vec4(srgb(base.xyz+value.xyz),base.w));
}
