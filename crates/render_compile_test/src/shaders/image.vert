#version 450

#extension GL_OES_standard_derivatives : require
precision highp float;

#import super::camera
#import super::ui_meterial

// 输入
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

// 输出
layout(location = 0) out vec2 vUv;
layout(location = 1) out vec2 vVertexPosition;

void main() {
	vVertexPosition = position;

	gl_Position = project * view * world * vec4(position.x, position.y, 1.0, 1.0);
	gl_Position.z = depth/60000.0;

	vUv = uv;
}