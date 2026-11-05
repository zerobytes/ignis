// Fragment position.w is inverse clip W; XY already contains viewport projection.
// Full viewport at origin; the UV parameterization is p.xy * 0.5 + 0.5.
fn screen_gradients(pixel:vec4<f32>, slope:vec2<f32>)->vec4<f32> {
 let u=g.matrix[0]+g.matrix[2]*slope.x;
 let v=g.matrix[1]+g.matrix[2]*slope.y;
 let ndc=vec2(pixel.x*2./g.material.y-1.,1.-pixel.y*2./g.material.z);
 let scale=vec2(g.material.y,-g.material.z)*0.5*pixel.w;
 let a=(u.xy-ndc*u.w)*scale;
 let b=(v.xy-ndc*v.w)*scale;
 let det=a.x*b.y-a.y*b.x;
 if abs(det)<1e-10 {return vec4(1.,0.,0.,1.);}
 return vec4(vec2(b.y,-a.y),vec2(-b.x,a.x))*(0.5/det);
}
