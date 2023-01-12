layout(set=0,binding=0) uniform M_0_0{
mat4 project;
mat4 view;
vec3 PATCH__0_0;
float f32;
};
layout(set=1,binding=0) uniform M_1_0{
mat4 world;
vec4 color;
vec3 PATCH__1_0;
float depth;
};
layout(set=2,binding=1)texture2D tex2d;
layout(set=2,binding=0) sampler samp;
layout(set=3,binding=1) uniform M_3_1{
vec4 dddd;
vec4 ffff;
vec3 eeee;
float PATCH__3_1;
};
layout(set=3,binding=0) uniform M_3_0{
vec4 aaaa;
vec4 cccc;
vec2 bbbb;
vec2 PATCH__3_0;
};
layout(location=0)in vec2 position;
layout(location=1)in vec2 color;
layout(location=0)out vec2 vVertexPosition;
#version 450

void main1(){

	gl_Position.z=depth/60000.+.2+bb;

}

void main(){
	vVertexPosition=position;

	gl_Position=project*view*world*vec4(position.x,position.y,1.,1.);
	gl_Position.z=depth/60000.+.2;

}
