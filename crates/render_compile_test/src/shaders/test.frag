#version 450

precision highp float;

layout(location=0)in vec2 vVertexPosition;

layout(location=0)out vec4 o_Target;

layout(set=1,binding=0)uniform ColorMaterial{
	mat4 world;
	@default(1.,2.,3.,4.)
	vec4 color;
	float depth;
};

#ifdef FFF
layout(set=1,binding=1)uniform ColorMaterial1{
	mat4 world1;
};
#endif

void main(){
	vec4 c=color.rgba;
	#ifdef GGG
	o_Target=c;
	#endif
}