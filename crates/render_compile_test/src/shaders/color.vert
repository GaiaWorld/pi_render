#version 450

#import super::ui_meterial

layout(location = 0) in vec2 position; // 输入位置

#ifdef VERT_COLOR
	layout(location = 1) in vec4 vertColor;
#endif

layout(location = 0) out vec2 vVertexPosition; // 输出位置

#ifdef VERT_COLOR
	layout(location = 1) out vec4 vColor; // 输出颜色
#endif

void main() {
	vVertexPosition = position;

	gl_Position = project * view * world * vec4(position.x, position.y, 1.0, 1.0);
	gl_Position.z = depth/60000.0;

	#ifdef VERT_COLOR
		vColor = vertColor;
	#endif
}