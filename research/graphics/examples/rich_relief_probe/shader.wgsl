struct Globals {matrix:mat4x4<f32>,eye:vec4<f32>,light:vec4<f32>,material:vec4<f32>};
@group(0) @binding(0) var<uniform> g:Globals;
@group(1) @binding(0) var colors:texture_2d<f32>;
@group(1) @binding(1) var filtering:sampler;
@group(1) @binding(2) var normals:texture_2d<f32>;
struct V {@builtin(position) clip:vec4<f32>,@location(0) p:vec3<f32>,@location(1) uv:vec2<f32>};
@vertex fn vs(@location(0) p:vec3<f32>)->V {var o:V;o.clip=g.matrix*vec4(p,1.);o.p=p;o.uv=p.xy*0.5+0.5;return o;}
@fragment fn fs(v:V,@builtin(front_facing) front:bool)->@location(0) vec4<f32> {
 let material=textureSample(colors,filtering,v.uv);let field=textureSample(normals,filtering,v.uv);
 let n=normalize(field.xyz*2.-1.)*select(-1.,1.,front);let l=g.light.xyz;let eye=normalize(g.eye.xyz-v.p);let h=normalize(l+eye);
 let rough=material.a;let power=mix(112.,8.,rough);let shine=mix(0.22,0.025,rough);
 let diffuse=0.25*field.a+0.9*max(dot(n,l),0.);let spec=shine*pow(max(dot(n,h),0.),power);
 return vec4(material.rgb*diffuse+vec3(spec),1.);
}
