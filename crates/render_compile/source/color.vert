#version 450

layout(location = 0) in vec2 position; // 输入位置

#ifdef VERT_COLOR
layout(location = 1) in vec4 color;
#endif

#ifdef SHADOW
	layout(location = 0) out vec2 vPosition; // 输出位置
#endif

#ifdef VERT_COLOR
	layout(location = 1) out vec4 vColor; // 输出颜色
#endif

layout(set = 0, binding = 0) uniform CameraMatrix {
	mat4 project;
	mat4 view;
};

layout(set = 1, binding = 0) uniform ColorMaterial {
	mat4 world;
	float depth;
	vec4 color;
	vec4 uRect; // xy是矩形最小点的坐标，zw是矩阵最大点的坐标；注：矩形必须排除阴影半径。
	float blur;
};

void main() {
	#ifdef SHADOW
		vPosition = position;
	#endif

	gl_Position = project * view * world * vec4(position.x, position.y, 1.0, 1.0);
	gl_Position.z = depth/60000.0;

	#ifdef VERT_COLOR
		vColor = color;
	#endif
}