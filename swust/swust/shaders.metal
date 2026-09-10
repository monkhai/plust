#include <metal_stdlib>
using namespace metal;

struct VertexOut {
    float4 position [[position]];
    float2 uv;
};

vertex VertexOut vertex_main(uint vid [[vertex_id]]) {
    float2 pos[3] = {
        float2(-1, -1),
        float2(3, -1),
        float2(-1, 3),
    };

    float2 uv[3] = {
        float2(0, 1),
        float2(2, 1),
        float2(0, -1),
    };

    VertexOut out;
    out.position = float4(pos[vid], 0, 1);
    out.uv = uv[vid];
    return out;
}

fragment float4 yuv_fragment(VertexOut in [[stage_in]],
                             texture2d<float> yTex [[texture(0)]],
                             texture2d<float> uvTex [[texture(1)]]) {
    
    float2 uv = float2(in.uv.x, in.uv.y);
    constexpr sampler s(address::clamp_to_edge, filter::linear);
    
    float2 uvSample = uvTex.sample(s, uv).rg;
    float y = yTex.sample(s, uv).r;
    
    float u = uvSample.r - 0.5;
    float v = uvSample.g - 0.5;
    
    float r = y + 1.5748 * v;
    float g = y - 0.1873 * u - 0.4681 * v;
    float b = y + 1.8556 * u;
    
    return float4(r, g, b, 1.0);
}
