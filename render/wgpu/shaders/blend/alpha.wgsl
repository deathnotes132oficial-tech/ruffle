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

    // SEM DESVIO, DE PROPOSITO.
    //
    // ALPHA multiplica a transparencia do destino pela da origem. Origem
    // totalmente transparente tem que dar resultado totalmente transparente —
    // e a conta abaixo ja faz isso sozinha quando src.a e zero.
    //
    // Aqui havia um "discard" nesse caso, que deixa o destino INTACTO. E o
    // oposto: em vez de apagar, preservava.
    //
    // No DDTank isso tapava a cratera. O jogo abre o buraco com ERASE, recorta
    // a borda queimada pelo terreno que sobrou usando ALPHA, e grava essa
    // borda de volta com mistura normal. Sem o recorte, a borda inteira e
    // opaca: ela era carimbada por cima do buraco, sumindo com ele na imagem
    // E na colisao, que le o mesmo BitmapData — dava pra andar no ar.
    return vec4<f32>(dst.rgb * src.a, src.a * dst.a);
}
