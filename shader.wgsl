// transform image coords (in pixels)
fn transform_coords(coords: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(coords.x, coords.y);
}
// transform 0.0-1.0 coords
fn transform_coords2(coords: vec2<f32>) -> vec2<f32> {
    return transform_coords(coords * data.img_dim) / data.img_dim;
}

// the rest is just boilerplate ignore it
struct Data {
    @location(0) img_dim: vec2<f32>,
    @location(1) win_dim: vec2<f32>,
    @location(2) pos: vec2<f32>,
    @location(3) scale: f32,
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
    var scale: vec2<f32>;
    let aspect = data.win_dim.x / data.win_dim.y * data.img_dim.y / data.img_dim.x;
    if (data.win_dim.y > data.win_dim.x) {
        scale = vec2<f32>(data.scale, data.scale * aspect);
    } else {
        scale = vec2<f32>(data.scale / aspect, data.scale);
    }
    out.pos = vec4<f32>(
        ((pos + data.pos) * 2.0 - 1) * scale,
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

@fragment
fn fs_main(inp: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(sampleClamp(texture, sampler1, transform_coords2(inp.tex_coords)).xyz / 2, 1.0);
}
