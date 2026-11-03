struct Camera {
    forward:vec4<f32>, right:vec4<f32>, camera_up:vec4<f32>, radial_up:vec4<f32>,
    shape:vec4<f32>, extent:vec4<f32>,
};
@group(0) @binding(0) var<uniform> camera:Camera;
@vertex fn vs_main(@builtin(vertex_index) vertex:u32)->@builtin(position) vec4<f32>{
    let x=f32((vertex<<1u)&2u);let y=f32(vertex&2u);
    return vec4<f32>(x*2.-1.,y*2.-1.,0.,1.);
}
@fragment fn fs_main(@builtin(position) pixel:vec4<f32>)->@location(0) vec4<f32>{
    let uv=pixel.xy/camera.extent.xy;
    let x=(2.*uv.x-1.)*camera.shape.z;let y=(1.-2.*uv.y)*camera.shape.w;
    let ray=normalize(camera.forward.xyz+x*camera.right.xyz+y*camera.camera_up.xyz);
    let b=camera.shape.y*dot(camera.radial_up.xyz,ray);
    let disc=b*b-camera.shape.x;
    let transverse=cross(camera.radial_up.xyz,ray);
    let inverse_distance=1./camera.shape.y;
    let far_hit=b<0. && dot(transverse,transverse)<=inverse_distance*inverse_distance;
    let hit=select(b<0. && disc>=0.,far_hit,camera.extent.w>0.5);
    if camera.extent.z>0.5 {return vec4<f32>(vec3<f32>(select(0.,1.,hit)),1.);}
    if !hit {return vec4<f32>(0.025,0.045,0.075,1.);}
    // Work in dimensionless units, with a stable near root. Source distances remain in meters.
    let distance=camera.shape.x/(-b+sqrt(max(disc,0.)));
    var normal=normalize(camera.radial_up.xyz*camera.shape.y+ray*distance);
    if camera.extent.w>0.5 {
        let closest=cross(ray,transverse)*camera.shape.y;
        normal=normalize(closest-ray*sqrt(max(0.,1.-dot(closest,closest))));
    }
    let light=0.35+0.65*max(dot(normal,normalize(vec3<f32>(0.4,0.8,-0.45))),0.);
    return vec4<f32>(vec3<f32>(0.055,0.24,0.36)*light,1.);
}
