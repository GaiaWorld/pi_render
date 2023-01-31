#version 450

layout(location=0)in vec2 position;// 输入位置

#ifdef AAA
layout(location=1)in vec2 color;// 输入位置
#endif

layout(location=0)out vec2 vVertexPosition;// 输出位置

#ifdef BBB
#import super::camera1
#endif

#ifdef CCC
layout(set=1,binding=0)uniform ColorMaterial{
	mat4 world;
	@default(1.,2.,3.,4.)
	vec4 color;
	float depth;
};
#endif

void main(){
	#ifdef DD
	vVertexPosition=position;
	#endif
	gl_Position=project*view*world*vec4(position.x,position.y,1.,1.);
	gl_Position.z=depth/60000.+.2;
	
}

void main1(){
	gl_Position.z=depth/60000.+.2+bb;
}