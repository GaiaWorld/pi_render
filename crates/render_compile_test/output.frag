layout(set=1,binding=1) uniform M_1_1{
mat4 world1;
};
layout(set=1,binding=0) uniform M_1_0{
mat4 world;
vec4 color;
vec3 PATCH__1_0;
float depth;
};
layout(location=0)in vec2 vVertexPosition;
layout(location=0)out vec4 o_Target;
#version 450

precision highp float;

void main(){
	vec4 c=color.rgba;

	o_Target=c;

}
