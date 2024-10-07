fn sineWave( p: vec2<f32> ) -> vec2<f32> {
    let pi = 3.14159;
    let A = 0.05;
    let w = 20.0 * pi;
    let t = 30.0*pi/180.0;
    let y = sin( w*p.x + t) * A;
    return vec2<f32>(p.x, p.y+y);
}

fn transform_coords(coords: vec2<f32>) -> vec2<f32> {
    return sineWave(coords); /*
    var coords1 = coords;
    if ((i32(coords.y * 100) % 3) == 0) {
        coords1.x = (coords1.x + 0.3) % 1.0;
    }
    if ((i32(coords.y * 100) % 3) == 1) {
        coords1.x = (coords1.x + 0.66) % 1.0;
    }
    return sineWave(coords1);*/
    // let coords = sineWave(coords1);
    // return coords; // vec2<f32>(coords.x * 6 - 2.5, coords.y * 4);
}

fn transform_coords2(coords: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(coords.x, abs((coords.y * 100) / (0.5 - coords.x)));
}

struct Data {
    @location(0) image_dim: vec2<f32>,
    @location(0) pos: vec2<f32>,
    @location(0) scale: f32,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let pos = vec2<f32>(f32((in_vertex_index & 2u) >> 1), f32(in_vertex_index & 1u));
    var out: VertexOutput;
    out.tex_coords = vec2<f32>(pos.x, 1 - pos.y);
    out.pos = vec4<f32>(
        ((pos.xy + data.pos) * data.scale + 0.5 * (2.0 - data.scale) - 1.0) * 2.0,
        0.0, 1.0
    );
    return out;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var sampler1: sampler;
@group(0) @binding(2)
var sampler2: sampler;
@group(0) @binding(3)
var<uniform> data: Data;

fn sampleClamp(texture: texture_2d<f32>, sampler1: sampler, v: vec2<f32>) -> vec4<f32> {
    if (v.x < 0.0 || v.x > 1.0 || v.y < 0.0 || v.y > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    return textureSample(texture, sampler1, v);
}
fn sampleClamp2(texture: texture_2d<f32>, sampler1: sampler, v: vec2<f32>) -> vec4<f32> {
    let v1 = vec2<f32>(v.x % 1.0, v.y % 1.0);
    return textureSample(texture, sampler1, v1);
}

@fragment
fn fs_main(inp: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(
        // sampleClamp2(texture, sampler1, transform_coords(inp.tex_coords)).xyz / 2
        // +
        sampleClamp2(texture, sampler1, transform_coords2(inp.tex_coords)).xyz / 2
        // sampleClamp2(texture, sampler1, inp.tex_coords).xyz / 2
        , 1.0);
}
