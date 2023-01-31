
#ifdef EEE
layout(set=0,binding=0)uniform CameraMatrix{
	mat4 project;
	mat4 view;
	float f32;
};
#endif

// 测试对齐问题
layout(set=3,binding=0)uniform Matrix1{
	vec4 aaaa;
	vec2 bbbb;
	vec4 cccc;
};

// 测试对齐问题
layout(set=3,binding=1)uniform Matrix2{
	vec4 dddd;
	vec3 eeee;
	vec4 ffff;
};

layout(set=3,binding=2)uniform vec2 arry1[3];

layout(set=2,binding=0)uniform sampler samp;
layout(set=2,binding=1)uniform texture2D tex2d;