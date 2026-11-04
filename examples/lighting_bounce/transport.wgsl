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
@group(0) @binding(7) var<storage,read_write> guides:array<u32>;
@group(0) @binding(8) var<storage,read_write> filtered:array<vec4<f32>>;
struct Work { counts:array<atomic<u32>,2>, ids:array<u32> }
@group(0) @binding(9) var<storage,read_write> work:Work;
@group(0) @binding(10) var<storage,read_write> indirect:array<u32>;
var<workgroup> low_scan:array<u32,64>;
var<workgroup> live_scan:array<u32,64>;
var<workgroup> low_base:u32;
var<workgroup> high_base:u32;
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
fn estimate_grid(p:vec3<f32>,n:vec3<f32>,cols:u32,rows:u32,seed:ptr<function,u32>)->vec3<f32> {
 var sum=vec3(0.);
 for(var j=0u;j<cols*rows;j++) {
  let v=(vec2(f32(j%cols),f32(j/cols))+pair(seed))/vec2(f32(cols),f32(rows));
  sum+=endpoint(p,cosine(n,v));
 }
 return sum/f32(cols*rows);
}
fn estimate(p:vec3<f32>,n:vec3<f32>,side:u32,seed:ptr<function,u32>)->vec3<f32> {
 return estimate_grid(p,n,side,side,seed);
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
 if u.energy.w>.5 {
  let b=boxes[h.obj];let half=(b.hi.xyz-b.lo.xyz)*.5;
  let edges=half-abs(h.p-(b.lo.xyz+b.hi.xyz)*.5);var risk=1e20;
  for(var k=0u;k<3u;k++){if abs(h.n[k])<.5{risk=min(risk,edges[k]);}}
  for(var j=0u;j<u.dims.z;j++){
   if j!=h.obj {risk=min(risk,distance(h.p,clamp(h.p,boxes[j].lo.xyz,boxes[j].hi.xyz)));}
  }
  receivers[i].rho.w=max(0.,risk);
 }
}
fn compatible(i:u32,j:u32)->bool {
 return receivers[i].p.w>0. && receivers[i].p.w==receivers[j].p.w && dot(receivers[i].n.xyz,receivers[j].n.xyz)>.999;
}
fn gaussian(dx:i32,dy:i32)->f32 {
 if dx==0 && dy==0{return 1.;}if dx!=0 && dy!=0{return .25;}return .5;
}
@compute @workgroup_size(8,8) fn guide(@builtin(global_invocation_id) id:vec3<u32>) {
 if id.x>=u.dims.x || id.y>=u.dims.y{return;}let i=id.y*u.dims.x+id.x;guides[i]=0u;
 if receivers[i].p.w==0.{return;}
 var pixel=1e20;
 for(var j=0;j<4;j++) {
  var offset=vec2<i32>(0,1);if j==1{offset=vec2(0,-1);}if j==2{offset=vec2(1,0);}if j==3{offset=vec2(-1,0);}
  let xy=vec2<i32>(id.xy)+offset;
  if any(xy<vec2(0)) || xy.x>=i32(u.dims.x) || xy.y>=i32(u.dims.y){continue;}
  let k=u32(xy.y)*u.dims.x+u32(xy.x);let d=distance(receivers[k].p.xyz,receivers[i].p.xyz);
  if compatible(i,k) && d>.000001{pixel=min(pixel,d);}
 }
 if pixel==1e20{return;}
 var mask=0u;var sw=0.;var sw2=0.;
 for(var dy=-1;dy<=1;dy++){for(var dx=-1;dx<=1;dx++){
  let xy=vec2<i32>(id.xy)+vec2(dx,dy);
  if any(xy<vec2(0)) || xy.x>=i32(u.dims.x) || xy.y>=i32(u.dims.y){continue;}
  let k=u32(xy.y)*u.dims.x+u32(xy.x);
  if compatible(i,k) && distance(receivers[k].p.xyz,receivers[i].p.xyz)<=2.5*pixel+.00001 {
   let bit=u32((dy+1)*3+dx+1);mask|=1u<<bit;let w=gaussian(dx,dy);sw+=w;sw2+=w*w;
  }
 }}
 let effective=sw*sw/max(sw2,.000001);
 if effective>=4.{mask|=512u;}if effective>=6.{mask|=1024u;}
 guides[i]=mask;
}
@compute @workgroup_size(8,8) fn reconstruct(@builtin(global_invocation_id) id:vec3<u32>) {
 if id.x>=u.dims.x || id.y>=u.dims.y{return;}let i=id.y*u.dims.x+id.x;
 let flag=select(512u,1024u,u.dims.w==17u);let mask=guides[i];var value=vec4(0.);
 if receivers[i].p.w>0. {
  value=result[i];
  if (mask&flag)!=0u {
   var sum=vec3(0.);var weight=0.;
   for(var dy=-1;dy<=1;dy++){for(var dx=-1;dx<=1;dx++){
    let bit=u32((dy+1)*3+dx+1);if (mask&(1u<<bit))==0u{continue;}
    let xy=vec2<i32>(id.xy)+vec2(dx,dy);let k=u32(xy.y)*u.dims.x+u32(xy.x);let w=gaussian(dx,dy);
    sum+=w*result[k].xyz;weight+=w;
   }}
   value=vec4(sum/weight,value.w);
  }
 }
 filtered[i]=value;let base=textureLoad(native,vec2<i32>(id.xy),0);
 textureStore(display,vec2<i32>(id.xy),vec4(srgb(base.xyz+value.xyz),base.w));
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
fn lookup(i:u32,gated:bool,fallback_side:u32,seed:ptr<function,u32>)->vec4<f32> {
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
 return vec4(estimate(origin,n,fallback_side,seed),select(1.,2.,fallback_side==8u));
}
// Frozen-field disagreement is a reliability feature, not an error certificate.
fn probe_spread(i:u32)->f32 {
 let origin=receivers[i].p.xyz+receivers[i].n.xyz*.002;
 let g=clamp((origin-u.grid.xyz)/u.grid.w,vec3(0.),vec3<f32>(u.counts.xyz-vec3(1u)));
 let cell=min(vec3<u32>(floor(g)),u.counts.xyz-vec3(2u));let k=axis_index(receivers[i].n.xyz);
 var lo=vec3(1e20);var hi=vec3(-1e20);var valid=false;
 for(var j=0u;j<8u;j++) {
  let bits=vec3(j&1u,(j>>1u)&1u,(j>>2u)&1u);let value=probes[index(cell+bits)*6u+k];
  if value.w>0.{lo=min(lo,value.xyz);hi=max(hi,value.xyz);valid=true;}
 }
 if !valid{return 1e20;}
 let range=hi-lo;return max(max(range.x,range.y),range.z)/max(max(max(u.energy.x,u.energy.y),u.energy.z),.000001);
}
// One global reservation per workgroup, never a CPU readback for dispatch sizes.
@compute @workgroup_size(1) fn reset_work(){atomicStore(&work.counts[0],0u);atomicStore(&work.counts[1],0u);}
@compute @workgroup_size(64) fn compact(@builtin(global_invocation_id) id:vec3<u32>,@builtin(local_invocation_index) lane:u32){
 let pixels=u.dims.x*u.dims.y;var live=0u;var low=0u;
 if id.x<pixels {if receivers[id.x].p.w>0.{live=1u;if (guides[id.x]&512u)!=0u{low=1u;}}}
 low_scan[lane]=low;live_scan[lane]=live;workgroupBarrier();
 for(var stride=1u;stride<64u;stride*=2u){
  var a=0u;var b=0u;if lane>=stride {a=low_scan[lane-stride];b=live_scan[lane-stride];}
  workgroupBarrier();low_scan[lane]+=a;live_scan[lane]+=b;workgroupBarrier();
 }
 if lane==0u {low_base=atomicAdd(&work.counts[0],low_scan[63]);high_base=atomicAdd(&work.counts[1],live_scan[63]-low_scan[63]);}
 workgroupBarrier();
 if live!=0u {
  if low!=0u{work.ids[low_base+low_scan[lane]-1u]=id.x;}
  else{work.ids[pixels+high_base+live_scan[lane]-low_scan[lane]-1u]=id.x;}
 }
}
@compute @workgroup_size(1) fn dispatch_args(){
 indirect[0]=(atomicLoad(&work.counts[0])+63u)/64u;indirect[1]=1u;indirect[2]=1u;
 indirect[3]=(atomicLoad(&work.counts[1])+63u)/64u;indirect[4]=1u;indirect[5]=1u;
}
// A real one-thread operation also gives reliable timestamp scope for empty lists.
@compute @workgroup_size(1) fn begin_batch(){indirect[5]=u.dims.w;}
fn sample_pixel(i:u32,cols:u32,rows:u32,flag:f32)->vec4<f32>{
 var seed=hash(i+u.counts.w*91237u+9923u);
 return vec4(receivers[i].rho.xyz*estimate_grid(receivers[i].p.xyz+receivers[i].n.xyz*.002,receivers[i].n.xyz,cols,rows,&seed),flag);
}
@compute @workgroup_size(64) fn sample_low(@builtin(global_invocation_id) id:vec3<u32>){
 if id.x>=atomicLoad(&work.counts[0]){return;}let i=work.ids[id.x];
 var cols=8u;if u.dims.w==21u{cols=5u;}if u.dims.w==22u{cols=6u;}if u.dims.w==23u{cols=7u;}
 result[i]=sample_pixel(i,cols,4u,0.);
}
@compute @workgroup_size(64) fn sample_high(@builtin(global_invocation_id) id:vec3<u32>){
 if id.x>=atomicLoad(&work.counts[1]){return;}let i=work.ids[u.dims.x*u.dims.y+id.x];result[i]=sample_pixel(i,8u,8u,2.);
}
fn srgb(c:vec3<f32>)->vec3<f32>{let x=max(c,vec3(0.));return select(1.055*pow(x,vec3(1./2.4))-.055,12.92*x,x<=vec3(.0031308));}
@compute @workgroup_size(8,8) fn shade(@builtin(global_invocation_id) id:vec3<u32>) {
 if id.x>=u.dims.x || id.y>=u.dims.y{return;}let i=id.y*u.dims.x+id.x;
 let mode=u.dims.w;var value=vec4(0.);
 if receivers[i].p.w>0. && mode!=0u {
  var seed=hash(i+u.counts.w*91237u+select(9923u,181331u,mode==7u || mode==8u));
  if mode>=17u {
   let eligible=(guides[i]&select(512u,1024u,mode==17u))!=0u;
   let origin=receivers[i].p.xyz+receivers[i].n.xyz*.002;
   if mode==18u || eligible {
    if mode==19u{value=vec4(estimate_grid(origin,receivers[i].n.xyz,8u,4u,&seed),0.);}
    else{value=vec4(estimate(origin,receivers[i].n.xyz,4u,&seed),0.);}
   }else{value=vec4(estimate(origin,receivers[i].n.xyz,8u,&seed),2.);}
  } else if mode>=9u {
   var threshold=select(select(.25,.5,mode==10u),.75,mode==11u);
   if mode==12u{threshold=1.;}
   if mode>=13u{threshold=.25;}
   if u.sun.w>0.{threshold=u.sun.w;}
   var direct=u.sun.w>=0. && receivers[i].rho.w<=threshold+.00001;
   if !direct && u.sun.w>=0. && mode>=13u {
    var limit=.02;if mode==14u{limit=.04;}if mode==15u{limit=.08;}if mode==16u{limit=.03;}
    direct=probe_spread(i)>limit-.000001;
   }
   if direct {
    value=vec4(estimate(receivers[i].p.xyz+receivers[i].n.xyz*.002,receivers[i].n.xyz,8u,&seed),2.);
   } else {value=lookup(i,true,8u,&seed);}
  } else if mode>=3u && mode<=6u {value=lookup(i,mode==4u || mode==6u,4u,&seed);}
  else {let side=select(select(4u,8u,mode==2u),64u,mode>=7u);
   value=vec4(estimate(receivers[i].p.xyz+receivers[i].n.xyz*.002,receivers[i].n.xyz,side,&seed),0.);
  }
  value=vec4(receivers[i].rho.xyz*value.xyz,value.w);
 }
 result[i]=value;if mode>=17u{return;}let base=textureLoad(native,vec2<i32>(id.xy),0);
 textureStore(display,vec2<i32>(id.xy),vec4(srgb(base.xyz+value.xyz),base.w));
}
