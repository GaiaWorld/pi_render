#version 450
// 输入uv
layout(location = 0) in vec2 vUv;
#ifdef VERTEX_COLOR
	layout(location = 1) in vec4 vColor;
#endif

// 输出颜色
layout(location = 0) out vec4 o_Target;

// set 1
layout(set = 1, binding = 0) uniform TextMaterial {
	mat4 world; // 世界矩阵
	float depth; // 深度
	// 纹理尺寸
	// 由于纹理纹理的尺寸会发生改变，一旦改变，每个文字的uv会随之而变
	// 如果纹理尺寸作为uniform传入着色器，文字uv采用绝对像素的方式描述，由着色器算出最终的uv
	// 当纹理尺寸发生改变时，每个文字渲染只需要修改TextureSize即可（TextureSize）是所用文字共用的，而无须再次为每个文字创建不同的uv buffer
	vec2 texture_size;
	vec4 uColor;
	vec4 strokeColor;
};

// set 2
layout (set = 2, binding = 0) uniform sampler samp;
layout (set = 2, binding = 1) uniform texture2D tex2d;

void main() {
	vec4 c = uColor;

	#ifdef VERTEX_COLOR
        c = vColor;
    #endif

	vec4 samp = texture(sampler2D(tex2d, samp), vUv);
    // samp.b < 1时，sample.b参与颜色计算，否则边缘过度部分可能出现锯齿
    // c.rgb = c.a * alpha * (samp.r * strokeColor.rgb + samp.g * c.rgb);
	// g表示填充分量
	// c.rgb = samp.g * c.rgb;

	// r表示描边分量
	#ifdef STROKE
		c.rgb = c.rgb * samp.g  + samp.r * strokeColor.rgb;
	#endif

	
	// 应该 c.a = 1.0 - samp.b, 由于纹理坐标误差， 导致采样到纹理的空白处（rgba都为0）， 会看到一条黑线
	// c.a = c.a * clamp(samp.a - samp.b, 0.0, 1.0);
	// b表示背景分量
	c.a = c.a * (1.0 - samp.b);

	

	o_Target = c;
	// o_Target = vec4(samp.r, samp.g, samp.b, 1.0);

	// o_Target = vec4(1.0, 0.0,0.0,1,0);
}