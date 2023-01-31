#version 450

#import super::ui_meterial
#import super::sdf

layout(location = 0) in vec2 vVertexPosition;

// 输入uv
layout(location = 1) in vec2 vUv;
#ifdef VERTEX_COLOR
	layout(location = 2) in vec4 vColor;
#endif

// 输出颜色
layout(location = 0) out vec4 o_Target;

// set 2
layout (set = 2, binding = 0) uniform sampler samp;
layout (set = 2, binding = 1) uniform texture2D tex2d;

void main() {
	vec4 c = color;

	#ifdef VERTEX_COLOR
        c = vColor;
    #endif

	vec4 samp = texture(sampler2D(tex2d, samp), vUv);
    // samp.b < 1时，sample.b参与颜色计算，否则边缘过度部分可能出现锯齿
    // c.rgb = c.a * alpha * (samp.r * strokeColorOrURect.rgb + samp.g * c.rgb);
	// g表示填充分量
	// c.rgb = samp.g * c.rgb;

	// r表示描边分量
	#ifdef STROKE
		c.rgb = c.rgb * samp.g  + samp.r * strokeColorOrURect.rgb;
	#endif

	
	// 应该 c.a = 1.0 - samp.b, 由于纹理坐标误差， 导致采样到纹理的空白处（rgba都为0）， 会看到一条黑线
	// c.a = c.a * clamp(samp.a - samp.b, 0.0, 1.0);
	// b表示背景分量
	c.a = c.a * (1.0 - samp.b);

	

	o_Target = c;
	// o_Target = vec4(1.0, 0.0, 0.0, 1.0);
	// o_Target = vec4(samp.r, samp.g, samp.b, 1.0);

	// o_Target = vec4(1.0, 0.0,0.0,1,0);
}