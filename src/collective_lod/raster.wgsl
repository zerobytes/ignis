// Diagnostic raster-compatible intersection. Grid is device-specific, not certified
// by the packet or selected automatically for unmeasured adapters.
fn collective_edge(a:vec2<f32>,b:vec2<f32>,p:vec2<f32>)->f32 {
    let d=b-a;let q=p-a;return d.x*q.y-d.y*q.x;
}
fn collective_top_left(a:vec2<f32>,b:vec2<f32>)->bool {
    return b.y<a.y || (b.y==a.y && b.x>a.x);
}
fn collective_screen(clip:vec4<f32>)->vec2<f32> {
    return (clip.xy/clip.w-collective_parameters.pixel_to_ndc.zw)/collective_parameters.pixel_to_ndc.xy;
}
fn collective_clip(i:CollectiveInstance,p:vec3<f32>)->vec4<f32> {
    let world=(i.world*vec4<f32>(p,1.0)).xyz;
    return globals.view_projection*vec4<f32>(world,1.0);
}
// A conservative local pad for screen snapping. Query bounds remain derived
// from the source; their runtime pad accounts for the chosen diagnostic grid.
fn collective_pad(i:CollectiveInstance,origin:vec3<f32>,lo:vec3<f32>,hi:vec3<f32>)->f32 {
    if i.control.y==0u {return 0.0;}
    let distance=length((lo+hi)*0.5-origin)+length((hi-lo)*0.5);
    return distance*abs(collective_parameters.pixel_to_ndc.y)*length(globals.atmosphere_up.xyz)*4.0/f32(i.control.y);
}
// xy are perspective-correct weights for source corners b/c. z is depth and
// w is positive eye depth (zero means absent). Coverage uses snapped vertices;
// interpolation stays unsnapped.
fn collective_raster(i:CollectiveInstance,triangle:u32,pixel:vec2<f32>)->vec4<f32> {
    let a=collective_clip(i,collective_triangle_vec(triangle));
    let b=collective_clip(i,collective_triangle_vec(triangle+3u));
    let c=collective_clip(i,collective_triangle_vec(triangle+6u));
    // CPU selection retains the complete source whenever its sphere crosses near.
    if min(a.w,min(b.w,c.w))<=0.05 {return vec4<f32>(0.0);}
    let pa=collective_screen(a);let pb=collective_screen(b);let pc=collective_screen(c);
    let grid=f32(i.control.y);
    let sa=round(pa*grid)/grid;let sb=round(pb*grid)/grid;let sc=round(pc*grid)/grid;
    let area=collective_edge(sa,sb,sc);
    if area==0.0 {return vec4<f32>(0.0);}
    let orientation=sign(area);
    let edges=vec3<f32>(collective_edge(sb,sc,pixel),collective_edge(sc,sa,pixel),collective_edge(sa,sb,pixel))*orientation;
    if any(edges<vec3<f32>(0.0)) {return vec4<f32>(0.0);}
    let top=vec3<bool>(select(collective_top_left(sc,sb),collective_top_left(sb,sc),area>0.0),select(collective_top_left(sa,sc),collective_top_left(sc,sa),area>0.0),select(collective_top_left(sb,sa),collective_top_left(sa,sb),area>0.0));
    if any((edges==vec3<f32>(0.0)) & !top) {return vec4<f32>(0.0);}
    let original_area=collective_edge(pa,pb,pc);
    if original_area==0.0 {return vec4<f32>(0.0);}
    let weights=vec3<f32>(collective_edge(pb,pc,pixel),collective_edge(pc,pa,pixel),collective_edge(pa,pb,pixel))/original_area;
    let perspective=weights/vec3<f32>(a.w,b.w,c.w);
    let bary=perspective/(perspective.x+perspective.y+perspective.z);
    let z=vec3<f32>(a.z/a.w,b.z/b.w,c.z/c.w);
    // Preserve a constant float depth plane, including its authored-order ties.
    let depth=z.x+weights.y*(z.y-z.x)+weights.z*(z.z-z.x);
    let eye=a.w+bary.y*(b.w-a.w)+bary.z*(c.w-a.w);
    return vec4<f32>(bary.y,bary.z,depth,eye);
}
