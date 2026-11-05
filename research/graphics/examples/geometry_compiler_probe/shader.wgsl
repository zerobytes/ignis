struct Globals {matrix:mat4x4<f32>,eye:vec4<f32>,light:vec4<f32>,material:vec4<f32>};
@group(0) @binding(0) var<uniform> g:Globals;
struct V {@builtin(position) clip:vec4<f32>,@location(0) p:vec3<f32>,@location(1) n:vec3<f32>};
@vertex fn vs(@location(0) p:vec3<f32>,@location(1) n:vec3<f32>)->V {var o:V;o.clip=g.matrix*vec4(p,1.);o.p=p;o.n=n;return o;}
@fragment fn fs(v:V,@builtin(front_facing) front:bool)->@location(0) vec4<f32> {
 let n=normalize(v.n)*select(-1.,1.,front);let l=g.light.xyz;let eye=normalize(g.eye.xyz-v.p);let h=normalize(l+eye);
 var color=vec3(0.32,0.26,0.19);var shine=0.12;var power=24.;
 if g.material.x>0.5 {color=vec3(0.17,0.26,0.3);shine=0.38;power=64.;}
 if g.material.x>1.5 {color=vec3(0.43,0.31,0.16);shine=0.7;power=128.;}
 // Continuous world-space material; identical for source and compiled meshes.
 let detail=0.92+0.08*sin(v.p.x*67.+v.p.z*23.)*sin(v.p.y*59.-v.p.z*17.);
 let diffuse=0.15+0.85*max(dot(n,l),0.);let spec=shine*pow(max(dot(n,h),0.),power);
 return vec4(color*detail*diffuse+vec3(spec),1.);
}
