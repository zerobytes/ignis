struct Settings {
    control: vec4<f32>, // strength, radius, start, end
    projection: vec4<f32>, // depth A/B, tan half-FOV x/y
    weights: array<vec4<f32>,4>,
};
@group(0) @binding(0) var color_image: texture_2d<f32>;
@group(0) @binding(1) var depth_image: texture_depth_2d;
@group(0) @binding(2) var<uniform> settings: Settings;

@vertex fn vs_main(@builtin(vertex_index) i:u32) -> @builtin(position) vec4<f32> {
    let p=array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0));
    return vec4<f32>(p[i],0.0,1.0);
}
fn axial_depth(d:f32)->f32 {
    if d>=1.0 {return 1000000.0;}
    return settings.projection.y/max(settings.projection.x-d,0.00000001);
}
fn influence(p:vec2<i32>,size:vec2<i32>,d:f32,z:f32)->f32 {
    if d>=1.0 {return 1.0;}
    let uv=(vec2<f32>(p)+0.5)/vec2<f32>(size)*2.0-1.0;
    let ray=vec3<f32>(uv*settings.projection.zw,1.0);
    return smoothstep(settings.control.z,settings.control.w,z*length(ray));
}
fn filtered(position:vec4<f32>,axis:vec2<i32>)->vec4<f32> {
    let p=vec2<i32>(position.xy);
    let size=vec2<i32>(textureDimensions(color_image));
    let base=textureLoad(color_image,p,0);
    let d=textureLoad(depth_image,p,0);
    let z=axial_depth(d);
    let fade=influence(p,size,d,z);
    if fade<=0.0 {return base;}
    let radius=i32(settings.control.y);
    var edge=false;
    for(var i=-radius;i<=radius;i++) {
        let q=clamp(p+axis*i,vec2<i32>(0),size-vec2<i32>(1));
        let nd=textureLoad(depth_image,q,0);
        if nd==d {continue;}
        let nz=axial_depth(nd);
        if ((d>=1.0)!=(nd>=1.0)) || abs(nz-z)>max(2.0,0.05*min(nz,z)) {edge=true;break;}
    }
    if !edge {return base;}
    var total=base.rgb;
    var weight_sum=1.0;
    for(var i=-radius;i<=radius;i++) {
        if i==0 {continue;}
        let q=clamp(p+axis*i,vec2<i32>(0),size-vec2<i32>(1));
        let nd=textureLoad(depth_image,q,0);
        let nz=axial_depth(nd);
        let k=u32(abs(i));
        let w=settings.weights[k/4u][k%4u];
        let f=min(fade,influence(q,size,nd,nz));
        // Nearby foreground never contributes a blurred halo to the background.
        total+=mix(base.rgb,textureLoad(color_image,q,0).rgb,f)*w;
        weight_sum+=w;
    }
    return vec4<f32>(mix(base.rgb,total/weight_sum,settings.control.x),base.a);
}
@fragment fn fs_horizontal(@builtin(position) p:vec4<f32>)->@location(0) vec4<f32> {
    return filtered(p,vec2<i32>(1,0));
}
@fragment fn fs_vertical(@builtin(position) p:vec4<f32>)->@location(0) vec4<f32> {
    return filtered(p,vec2<i32>(0,1));
}
