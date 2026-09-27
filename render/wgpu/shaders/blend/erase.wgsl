// NOTE: The `common.wgsl` source is prepended to this before compilation.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(2) @binding(0) var parent_texture: texture_2d<f32>;
@group(2) @binding(1) var current_texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

@vertex
fn main_vertex(in: common__VertexInput) -> VertexOutput {
    let pos = vec4<f32>((in.position.x * 2.0) - 1.0, -(in.position.y * 2.0) + 1.0, 0.0, 1.0);
    let uv = vec2<f32>(in.position.xy);
    return VertexOutput(pos, uv);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // dst is the parent pixel we're blending onto
    var dst: vec4<f32> = textureSample(parent_texture, texture_sampler, in.uv);
    // src is the pixel that we want to apply
    var src: vec4<f32> = textureSample(current_texture, texture_sampler, in.uv);

    if (src.a > 0.0) {
        return vec4<f32>(dst.rgb * (1.0 - src.a), (1.0 - src.a) * dst.a);
    } else {
        // EXPERIMENTO — cratera do DDTank.
        //
        // Aqui havia um "discard". Ele e equivalente a devolver o proprio
        // pixel do mapa, com uma diferenca: descarte com varias amostras por
        // pixel decide amostra por amostra, e a transparencia final vira a
        // media delas. Devolvendo o valor direto, o resultado nao depende
        // disso.
        return dst;
    }
}
