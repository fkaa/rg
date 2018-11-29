struct VSInput {
    float2 pos : POSITION;
    float2 uv : TEXCOORD;
    float4 color: COLOR;
};

struct PSInput {
    float4 pos : SV_Position;
    float2 uv : TEXCOORD;
    float4 color : COLOR;
};

cbuffer Camera : register(b0)
{
    float4x4 Projection;
}

PSInput VS(VSInput input)
{
    PSInput output;

    output.pos = mul(Projection, float4(input.pos, 0.f, 1.f));
    output.uv = input.uv;
    output.color = input.color;

    return output;
}

Texture2D Texture : register(t0);
SamplerState Sampler : register(s0);

float4 PS(PSInput input) : SV_Target0 {
    return float4(input.color.rgb, Texture.Sample(Sampler, input.uv).r * input.color.a);
}
