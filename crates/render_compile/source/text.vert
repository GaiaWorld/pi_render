#version 450

// 输入
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
#ifdef VERTEX_COLOR
	layout(location = 2) in vec4 color; // 顶点色
#endif

// 输出
layout(location = 0) out vec2 vUv;
#ifdef VERTEX_COLOR
	layout(location = 1) out vec4 vColor;
#endif

// set 0
layout(set = 0, binding = 0) uniform CameraMatrix {
	mat4 project;
	mat4 view;
};

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

void main() {
	vec4 p = view * world * vec4(position.x, position.y, 1.0, 1.0);
	gl_Position = project * vec4(floor(p.x + 0.5 ), floor(p.y + 0.5), 1.0, 1.0);
	gl_Position.z = depth/60000.0;

	vUv = uv/texture_size;
	#ifdef VERTEX_COLOR
		vColor = color;
	#endif
}