#version 450

#ifdef SHADOW
	layout(location = 0) in vec2 vPosition;
#endif

#ifdef VERT_COLOR
	layout(location = 1) in vec4 vColor;
#endif

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 0) uniform ColorMaterial {
	mat4 world; // 世界矩阵
	float depth; // 深度
	
	vec4 color; // 颜色
	vec4 uRect; // 阴影范围, xy是矩形最小点的坐标，zw是矩阵最大点的坐标；注：矩形必须排除阴影半径。
	float blur;	// 阴影模糊半径
	
};

// 从webrender-15版本中拷贝过来的shader

// An approximation of the error function, which is related to the integral of the Gaussian
// function:
//
//     "erf"(x) = 2/sqrt(pi) int_0^x e^(-t^2) dt
//              ~~ 1 - 1 / (1 + a_1 x + a_2 x^2 + a_3 x^3 + a_4 x^4)^4
//
// where:
//
//     a_1 = 0.278393, a_2 = 0.230389, a_3 = 0.000972, a_4 = 0.078108
//
// This approximation is accurate to '5 xx 10^-4', more than accurate enough for our purposes.
//
// See: https://en.wikipedia.org/wiki/Error_function#Approximation_with_elementary_functions
float erf(float x) {
	bool negative = x < 0.0;
	if (negative)
		x = -x;
	float x2 = x * x;
	float x3 = x2 * x;
	float x4 = x2 * x2;
	float denom = 1.0 + 0.278393 * x + 0.230389 * x2 + 0.000972 * x3 + 0.078108 * x4;
	float result = 1.0 - 1.0 / (denom * denom * denom * denom);
	return negative ? -result : result;
}

// A useful helper for calculating integrals of the Gaussian function via the error function:
//
//      "erf"_sigma(x) = 2 int 1/sqrt(2 pi sigma^2) e^(-x^2/(2 sigma^2)) dx
//                     = "erf"(x/(sigma sqrt(2)))
float erfSigma(float x, float sigma) {
	return erf(x / (sigma * 1.4142135623730951));
}

// Returns the blurred color value from the box itself (not counting any rounded corners). 'p_0' is
// the vector distance to the top left corner of the box; 'p_1' is the vector distance to its
// bottom right corner.
//
//      "colorFromRect"_sigma(p_0, p_1)
//          = int_{p_{0_y}}^{p_{1_y}} int_{p_{1_x}}^{p_{0_x}} G_sigma(y) G_sigma(x) dx dy
//          = 1/4 ("erf"_sigma(p_{1_x}) - "erf"_sigma(p_{0_x}))
//              ("erf"_sigma(p_{1_y}) - "erf"_sigma(p_{0_y}))
float colorFromRect(vec2 p0, vec2 p1, float sigma) {
	return (erfSigma(p1.x, sigma) - erfSigma(p0.x, sigma)) *
		(erfSigma(p1.y, sigma) - erfSigma(p0.y, sigma)) / 4.0;
}

// The blurred color value for the point at 'pos' with the top left corner of the box at
// 'p_{0_"rect"}' and the bottom right corner of the box at 'p_{1_"rect"}'.
float getShadowAlpha(vec2 pos, vec2 ptMin, vec2 ptMax, float sigma) {
	// Compute the vector distances 'p_0' and 'p_1'.
	vec2 dMin = pos - ptMin, dMax = pos - ptMax;

	// Compute the basic color '"colorFromRect"_sigma(p_0, p_1)'. This is all we have to do if
	// the box is unrounded.
	// return pos.x/(ptMax.x - ptMin.x);
	return colorFromRect(dMin, dMax, sigma);
	// return 0.5;
}

void main() {
	vec4 c = color.rgba;

	// 顶点色
	#ifdef VERT_COLOR
		c = vColor;
	#endif

	#ifdef SHADOW
		c.a = c.a * getShadowAlpha(vPosition, uRect.xy, uRect.zw, blur / 2.0);
	#endif

	o_Target = c;
	// o_Target = vec4( getShadowAlpha(vPosition, uRect.xy, uRect.zw, blur / 2.0), 0.0, 0.0, 1.0);
	// o_Target = vec4(color.rgb, 0.5);
	// o_Target = vec4( getShadowAlpha(vPosition, uRect.xy, uRect.zw, blur / 2.0), 0.0, 0.0, 1.0);
	// o_Target = vec4(vPosition.x, vPosition.y, 0.0, 1.0);
	// o_Target = vec4(uRect.x, uRect.y, uRect.z, uRect.w);
}