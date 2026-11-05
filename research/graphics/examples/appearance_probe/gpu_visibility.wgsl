struct Globals {matrix:mat4x4<f32>,inverse:mat4x4<f32>,eye:vec4<f32>,light:vec4<f32>,screen:vec4<f32>};
struct Instance {position:vec4<f32>,info:vec4<u32>};
struct Node {lo:vec4<f32>,hi:vec4<f32>,info:vec4<u32>};
struct Command {indices:u32,instances:u32,first:u32,base:i32,instance:u32};
struct Settings {regions:u32,mode:u32,levels:u32,occluders:u32};
@group(0) @binding(0) var<uniform> g:Globals;
@group(0) @binding(1) var<storage,read> input:array<Instance>;
@group(0) @binding(2) var<storage,read> nodes:array<Node>;
@group(0) @binding(3) var pyramid:texture_2d<f32>;
@group(0) @binding(4) var<storage,read_write> output:array<Instance>;
@group(0) @binding(5) var<storage,read_write> commands:array<Command>;
@group(0) @binding(6) var<storage,read_write> alive:array<u32>;
@group(0) @binding(7) var<storage,read> members:array<u32>;
@group(0) @binding(8) var<uniform> settings:Settings;
// A convex opaque box casts a convex shadow volume from a point camera.
// Every enlarged source corner must be hidden by the SAME shrunken occluder.
fn segment_hits(point:vec3<f32>,lo:vec3<f32>,hi:vec3<f32>)->bool {
 let delta=point-g.eye.xyz;var enter=0.0;var leave=1.0;
 for(var a=0u;a<3u;a++){
  if abs(delta[a])<0.00000001 {if g.eye[a]<=lo[a]||g.eye[a]>=hi[a]{return false;}}
  else {let t0=(lo[a]-g.eye[a])/delta[a];let t1=(hi[a]-g.eye[a])/delta[a];enter=max(enter,min(t0,t1));leave=min(leave,max(t0,t1));}
 }
 return enter<leave && enter<0.99999;
}
fn analytic_hidden(n:Node,o:Instance)->bool {
 let margin=vec3<f32>(0.0001*o.position.w+0.00001*max(max(abs(o.position.x),abs(o.position.y)),abs(o.position.z)));
 let source_lo=n.lo.xyz*o.position.w+o.position.xyz-margin;let source_hi=n.hi.xyz*o.position.w+o.position.xyz+margin;
 for(var wall=0u;wall<settings.occluders;wall++){
  let box=nodes[settings.regions+3u+wall];let inst=input[97u+wall];let lo=box.lo.xyz+inst.position.xyz+0.002;let hi=box.hi.xyz+inst.position.xyz-0.002;
  if all(g.eye.xyz>=lo)&&all(g.eye.xyz<=hi){continue;}
  var hidden=true;
  for(var corner=0u;corner<8u;corner++){
   let p=mix(source_lo,source_hi,vec3<f32>(f32(corner&1u),f32((corner>>1u)&1u),f32((corner>>2u)&1u)));
   if !segment_hits(p,lo,hi){hidden=false;break;}
  }
  if hidden{return true;}
 }
 return false;
}
// Return true for uncertainty, including a box crossing the near plane.
fn survives(n:Node,o:Instance,occlusion:bool)->bool {
 var outside=63u;var near=false;var lo=vec2<f32>(1e30);var hi=vec2<f32>(-1e30);var closest=0.0;
 for(var i=0u;i<8u;i++){
  let p=mix(n.lo.xyz,n.hi.xyz,vec3<f32>(f32(i&1u),f32((i>>1u)&1u),f32((i>>2u)&1u)))*o.position.w+o.position.xyz;
  let q=g.matrix*vec4<f32>(p,1);var mask=0u;
  if q.x < -q.w {mask|=1u;}if q.x>q.w {mask|=2u;}if q.y < -q.w {mask|=4u;}if q.y>q.w {mask|=8u;}if q.z<0.0 {mask|=16u;}if q.z>q.w {mask|=32u;}
  outside&=mask;
  if q.w<=0.1001 {near=true;} else {let ndc=q.xyz/q.w;lo=min(lo,ndc.xy);hi=max(hi,ndc.xy);closest=max(closest,ndc.z);}
 }
 if outside!=0u {return false;}
 if near||!occlusion {return true;}
 if settings.mode>=4u {return !analytic_hidden(n,o);}
 // Outward pixel rounding, two pixels of support, and a positive depth margin.
 let lower=clamp(floor(vec2<f32>(lo.x*0.5+0.5,0.5-hi.y*0.5)*g.screen.xy)-2.0,vec2<f32>(0),g.screen.xy-1.0);
 let upper=clamp(ceil(vec2<f32>(hi.x*0.5+0.5,0.5-lo.y*0.5)*g.screen.xy)+2.0,vec2<f32>(0),g.screen.xy-1.0);
 let span=max(upper.x-lower.x+1.0,upper.y-lower.y+1.0);
 let level=min(u32(max(0.0,ceil(log2(max(span/4.0,1.0))))),settings.levels-1u);
 let cell=f32(4u<<level);let a=vec2<i32>(floor(lower/cell));let b=vec2<i32>(floor(upper/cell));
 var farthest=1.0;
 for(var y=a.y;y<=b.y;y++){for(var x=a.x;x<=b.x;x++){farthest=min(farthest,textureLoad(pyramid,vec2<i32>(x,y),i32(level)).x);}}
 return closest+0.000002>=farthest;
}
var<workgroup> flags:array<u32,32>;
@compute @workgroup_size(32) fn roots(@builtin(workgroup_id) group:vec3<u32>,@builtin(local_invocation_index) lane:u32){
 let asset=group.x;let id=members[asset*32u+lane];let n=nodes[settings.regions+asset];let o=input[id];
 let yes=survives(n,o,settings.mode>=2u);flags[lane]=select(0u,1u,yes);alive[id-1u]=flags[lane];workgroupBarrier();
 var prefix=0u;for(var i=0u;i<lane;i++){prefix+=flags[i];}
 if yes {output[asset*96u+prefix]=o;}
 if lane==31u {commands[asset]=Command(n.info.y,prefix+flags[lane],n.info.x,0i,asset*96u);}
}
@compute @workgroup_size(32) fn regions(@builtin(workgroup_id) group:vec3<u32>,@builtin(local_invocation_index) lane:u32){
 let region=group.x;let n=nodes[region];let id=members[n.info.z*32u+lane];let o=input[id];
 var yes=false;if alive[id-1u]!=0u {yes=survives(n,o,settings.mode==3u||settings.mode==5u);}
 flags[lane]=select(0u,1u,yes);workgroupBarrier();var prefix=0u;for(var i=0u;i<lane;i++){prefix+=flags[i];}
 let start=(3u+region)*96u;
 if yes {output[start+prefix]=o;}
 if lane==31u {commands[3u+region]=Command(n.info.y,prefix+flags[lane],n.info.x,0i,start);}
}
