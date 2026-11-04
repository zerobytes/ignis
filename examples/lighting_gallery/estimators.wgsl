// Laboratory AO only: cosine measure, 4m visibility, exact 4x4 cache integral.
struct Params { inv:mat4x4<f32>, eye:vec4<f32>, dims:vec4<u32>, light:vec4<f32> }
struct Box { lo:vec4<f32>, hi:vec4<f32>, color:vec4<f32> }
struct Receiver { p:vec4<f32>, n:vec4<f32>, color:vec4<f32>, stats:vec4<f32>, watch:vec4<f32>, values:vec4<f32>, cells:array<f32,16> }
@group(0) @binding(0) var<uniform> u:Params;
@group(0) @binding(1) var<storage,read> boxes:array<Box>;
@group(0) @binding(2) var<storage,read_write> r:array<Receiver>;
@group(0) @binding(3) var native:texture_2d<f32>;
@group(0) @binding(4) var output:texture_storage_2d<rgba8unorm,write>;
@group(0) @binding(5) var<storage,read_write> estimates:array<f32>;
fn hash(x:u32)->u32 { var v=x; v=(v^(v>>16u))*0x7feb352du;v=(v^(v>>15u))*0x846ca68bu;return v^(v>>16u); }
fn random(seed:ptr<function,u32>)->f32 { *seed=hash(*seed+0x9e3779b9u);return f32(*seed>>8u)/16777216.; }
fn uv(seed:ptr<function,u32>)->vec2<f32> { let a=random(seed);let b=random(seed);return vec2(a,b); }
fn intersect(p:vec3<f32>,d:vec3<f32>,b:Box)->vec2<f32> {
 let a=(b.lo.xyz-p)/d;let c=(b.hi.xyz-p)/d;
 let lo=min(a,c);let hi=max(a,c);
 return vec2(max(max(lo.x,lo.y),lo.z),min(min(hi.x,hi.y),hi.z));
}
fn visible(i:u32,v:vec2<f32>)->f32 {
 let n=r[i].n.xyz;
 let t=normalize(cross(select(vec3(0.,1.,0.),vec3(1.,0.,0.),abs(n.y)>.9),n));
 let d=t*(sqrt(v.x)*cos(6.283185307*v.y))+cross(n,t)*(sqrt(v.x)*sin(6.283185307*v.y))+n*sqrt(1.-v.x);
 let p=r[i].p.xyz+n*.002;
 for(var j=0u;j<u.dims.z;j++) { let h=intersect(p,d,boxes[j]);if h.y>=max(h.x,0.) && h.x<4. && h.y>.001 {return 0.;} }
 return 1.;
}
fn g(i:u32,v:vec2<f32>)->f32 { let c=min(vec2<u32>(v*4.),vec2(3u));return r[i].cells[c.y*4u+c.x]; }
fn learn(i:u32,seed:ptr<function,u32>)->f32 {
 var sf=0.;var sg=0.;var sg2=0.;var sfg=0.;
 for(var j=0u;j<4u;j++){let v=uv(seed);let f=visible(i,v);let c=g(i,v);sf+=f;sg+=c;sg2+=c*c;sfg+=f*c;}
 let den=sg2-sg*sg/4.;if den<.0001{return 0.;}return clamp((sfg-sf*sg/4.)/den,0.,2.);
}
@compute @workgroup_size(8,8) fn setup(@builtin(global_invocation_id) id:vec3<u32>) {
 if id.x>=u.dims.x || id.y>=u.dims.y {return;}let i=id.y*u.dims.x+id.x;
 let xy=vec2((f32(id.x)+.5)/f32(u.dims.x)*2.-1.,1.-(f32(id.y)+.5)/f32(u.dims.y)*2.);
 let q=u.inv*vec4(xy,0.,1.);let d=normalize(q.xyz/q.w-u.eye.xyz);
 var nearest=1e20;var obj=0u;
 for(var j=0u;j<u.dims.z;j++){let h=intersect(u.eye.xyz,d,boxes[j]);if h.x>0. && h.y>=h.x && h.x<nearest{nearest=h.x;obj=j;}}
 r[i].p=vec4(0.);if nearest==1e20{return;}
 let p=u.eye.xyz+nearest*d;let b=boxes[obj];let center=(b.lo.xyz+b.hi.xyz)*.5;let local=(p-center)/((b.hi.xyz-b.lo.xyz)*.5);
 var n=vec3(0.,0.,sign(local.z));if abs(local.x)>abs(local.y) && abs(local.x)>abs(local.z){n=vec3(sign(local.x),0.,0.);}else if abs(local.y)>abs(local.z){n=vec3(0.,sign(local.y),0.);}
 r[i].p=vec4(p,1.);r[i].n=vec4(n,0.);r[i].color=b.color;r[i].stats=vec4(0.);r[i].values=vec4(0.);
 if u.dims.w>=2u && u.dims.w<=8u {
 var sum=0.;for(var j=0u;j<16u;j++){let v=(vec2(f32(j%4u),f32(j/4u))+.5)/4.;let f=visible(i,v);r[i].cells[j]=f;sum+=f;}r[i].stats.x=sum/16.;
 }
}
fn srgb(c:vec3<f32>)->vec3<f32>{let x=max(c,vec3(0.));return select(1.055*pow(x,vec3(1./2.4))-.055,12.92*x,x<=vec3(.0031308));}
@compute @workgroup_size(8,8) fn shade(@builtin(global_invocation_id) id:vec3<u32>) {
 if id.x>=u.dims.x || id.y>=u.dims.y {return;}let i=id.y*u.dims.x+id.x;let mode=u.dims.w;let frame=u32(u.light.y);
 var ao=1.;
 if r[i].p.w>0. && mode!=0u {
 var train=hash(i+frame*91237u+771u);var y=hash(i+frame*91237u+select(9923u,181331u,mode==9u));var a=1.;var budget=16u;
 if mode==2u {ao=r[i].stats.x;}else{
 if mode==1u || mode==9u {a=0.;}if mode==9u{budget=512u;}
 let constant=r[i].stats.x==0. || r[i].stats.x==1.;
 if mode>=4u && mode<=8u && constant {a=0.;}
 if (mode==4u || mode==5u) && !constant {a=learn(i,&train);budget-=4u;
 if mode==5u && a>0. {var sf=0.;var se=0.;var sf2=0.;var se2=0.;for(var j=0u;j<4u;j++){let v=uv(&train);let f=visible(i,v);let e=f-a*g(i,v);sf+=f;se+=e;sf2+=f*f;se2+=e*e;}budget-=4u;let vf=max(0.,sf2-sf*sf/4.);let ve=max(0.,se2-se*se/4.);if vf<.0001 || ve/vf>=.9*f32(budget)/16.{a=0.;}}
 }
 if mode>=6u && mode<=8u && !constant {
 var refresh=frame%32u==0u || (mode==7u && r[i].stats.z!=u.light.z);
 if mode==8u {
 if frame%4u==0u {var watch=hash(i+431u);let v0=uv(&watch);let v1=uv(&watch);let f0=visible(i,v0);let f1=visible(i,v1);budget-=2u;
 if frame>0u && (f0!=r[i].values.x || f1!=r[i].values.y){refresh=true;}
 // Fixed paired directions within an epoch, checked every 4 frames.
 r[i].watch=vec4(v0,v1);r[i].values=vec4(f0,f1,0.,0.);
 }
 }
 if refresh {r[i].stats.y=learn(i,&train);r[i].stats.z=u.light.z;budget-=4u;}a=r[i].stats.y;
 }
 var sum=0.;for(var j=0u;j<budget;j++){let v=uv(&y);let f=visible(i,v);var c=0.;if mode>=2u && mode<=8u{c=g(i,v);}sum+=f-a*c;}
 ao=a*r[i].stats.x+sum/f32(budget);
 }
 }
 estimates[i]=ao;let base=textureLoad(native,vec2<i32>(id.xy),0);
 let color=base.xyz+r[i].color.xyz*u.light.x*(ao-1.);
 textureStore(output,vec2<i32>(id.xy),vec4(srgb(color),base.w));
}
