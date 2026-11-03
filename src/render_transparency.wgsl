@group(0) @binding(0) var background: texture_2d<f32>;
@group(0) @binding(1) var accumulation: texture_2d<f32>;
@group(0) @binding(2) var transmission: texture_2d<f32>;
@group(0) @binding(3) var additive: texture_2d<f32>;
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(uv*2.0-1.0, 0.0, 1.0);
}
@fragment fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    let p = vec2<i32>(pixel.xy);
    // fp16 overdraw can saturate. Clamp both numerator and denominator before
    // normalization so extreme recipes never turn infinity/infinity into NaN.
    let a = min(textureLoad(accumulation, p, 0), vec4<f32>(65504.0));
    let t = clamp(textureLoad(transmission, p, 0), vec4<f32>(0.0), vec4<f32>(1.0));
    let e = min(textureLoad(additive, p, 0).rgb, vec3<f32>(65504.0));
    let opaque = textureLoad(background, p, 0).rgb;
    let surface = a.rgb / max(a.a, 0.000001) * (1.0-t.a);
    // Additive radiance is treated as behind the accumulated sheets.
    return vec4<f32>(surface + (opaque+e)*t.rgb, 1.0);
}
