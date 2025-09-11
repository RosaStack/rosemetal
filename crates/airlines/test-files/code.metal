#include <metal_stdlib>
using namespace metal;

constant float2 position[3] = { float2(0.0, -0.5), float2(0.5, 0.5), float2(-0.5, 0.5) };
constant float3 color[3] = { float3(1.0, 0.0, 0.0), float3(0.0, 1.0, 0.0), float3(0.0, 0.0, 1.0) };

struct FinalOutput {
	float3 fragColor [[user(locn0)]];
	float4 mtlPosition [[position]];
};

vertex FinalOutput vertexMain(uint vertexID [[vertex_id]]) {
	FinalOutput out = { 
		.mtlPosition = float4(position[vertexID], 0.0, 1.0), 
		.fragColor = color[vertexID] };

	return out;
}