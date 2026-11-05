// Heightfield diagnostic: chart UV = .5 * world XY + .5.
// The compiled shading normal approximates the geometric tangent plane.
// Return the UV footprint per physical pixel, independent of quad derivatives.
fn analytic_footprint(p:vec3<f32>, n:vec3<f32>)->f32 {
 let clip=g.matrix*vec4(p,1.);
 let u=g.matrix*vec4(n.z,0.,-n.x,0.);
 let v=g.matrix*vec4(0.,n.z,-n.y,0.);
 let ndc=clip.xy/clip.w;
 let scale=vec2(g.material.y,-g.material.z)*0.5/clip.w;
 let a=(u.xy-ndc*u.w)*scale;
 let b=(v.xy-ndc*v.w)*scale;
 let det=a.x*b.y-a.y*b.x;
 // A singular tangent projection has an unbounded footprint: use the last mip.
 if abs(det)<1e-10 {return 1.;}
 let dx=vec2(b.y,-a.y)*(0.5*n.z/det);
 let dy=vec2(-b.x,a.x)*(0.5*n.z/det);
 return clamp(max(length(dx),length(dy)),1e-10,1.);
}
