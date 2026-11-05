// IGT1 source-only heightfield; one vec2 dz/dx,dz/dy per indexed triangle.
@group(2) @binding(0) var<storage,read> slopes:array<vec2<f32>>;
fn geometric_gradients(p:vec3<f32>, slope:vec2<f32>)->vec4<f32> {
 let clip=g.matrix*vec4(p,1.);
 let u=g.matrix*vec4(1.,0.,slope.x,0.);
 let v=g.matrix*vec4(0.,1.,slope.y,0.);
 let ndc=clip.xy/clip.w;
 let scale=vec2(g.material.y,-g.material.z)*0.5/clip.w;
 let a=(u.xy-ndc*u.w)*scale;
 let b=(v.xy-ndc*v.w)*scale;
 let det=a.x*b.y-a.y*b.x;
 if abs(det)<1e-10 {return vec4(1.,0.,0.,1.);}
 return vec4(vec2(b.y,-a.y),vec2(-b.x,a.x))*(0.5/det);
}
