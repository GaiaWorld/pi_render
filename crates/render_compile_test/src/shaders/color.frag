#version 450

precision highp float;

#import super::ui_meterial
#import super::sdf

layout(location = 0) in vec2 vVertexPosition;

#ifdef VERT_COLOR
	layout(location = 1) in vec4 vColor;
#endif

layout(location = 0) out vec4 o_Target;
	
float sdfEllipse(vec2 pt, vec2 center, vec2 ab)
{
	pt -= center;
	
	// 求 (1/a, 1/b)
	vec2 recAB = 1.0 / ab;
	// 求 (x/a, y/b) = (x, y) * (1/a, 1/b)
	vec2 scale = pt * recAB;
	
	// 椭圆值 f = (x/a)^2 + (y/b)^2 - 1
	return dot(scale, scale) - 1.0;
}

float sdfRect(vec2 pt, vec2 wh)
{
	vec2 d = abs(pt) - wh;
	return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
}

float cross_pt(vec2 v1, vec2 v2) {
	return -(v1.x * v2.y - v1.y * v2.x);
}
// p0, p1, p2 是否 逆时针
bool is_ccw(vec2 p0, vec2 p1, vec2 p2) {
	vec2 v1 = p1 - p0;
	vec2 v2 = p2 - p0;
	float r = cross_pt(v1, v2);
	return r > 0.0;
}
bool is_left_top(vec2 pt, vec2 wh, vec2 center) {
	vec2 pt0 = vec2(-wh.x, center.y);
	vec2 pt1 = vec2(center.x, -wh.y);
	return is_ccw(pt, pt0, pt1);
}
bool is_top_right(vec2 pt, vec2 wh, vec2 center) {
	vec2 pt0 = vec2(center.x, -wh.y);
	vec2 pt1 = vec2(wh.x, center.y);
	return is_ccw(pt, pt0, pt1);
}
bool is_right_bottom(vec2 pt, vec2 wh, vec2 center) {
	vec2 pt0 = vec2(wh.x, center.y);
	vec2 pt1 = vec2(center.x, wh.y);
	return is_ccw(pt, pt0, pt1);
}
bool is_bottom_left(vec2 pt, vec2 wh, vec2 center) {
	vec2 pt0 = vec2(center.x, wh.y);
	vec2 pt1 = vec2(-wh.x, center.y);
	return is_ccw(pt, pt0, pt1);
}

 // 根据 d, 抗锯齿, 返回 alpha值
float antialiase(float d) 
{
	float anti = 1.0 * fwidth(d);
	
	// smoothstep(-a, a, d) 意思是 根据 d-值 将 [-a, a] 平滑到 [0, 1] 中
	// d < -a, 全内部, 得到0, 这时期望 alpha = 1.0
	// d > a, 全外部, 得到1, 这时期望 alpha = 0.0
	
	return 1.0 - smoothstep(-anti, anti, d);
}

#ifdef BORDER_RADIUS
	float antialiase_round_rect(vec2 pt, vec2 extent, vec2 offset1, vec2 offset2, vec2 offset3, vec2 offset4) {
		float d_rect = sdfRect(pt, extent);
		float a_rect = antialiase(d_rect);
		vec2 center = vec2(-extent.x + offset1.x, -extent.y + offset1.y); 
		if (is_left_top(pt, extent, center)) {
			float d = sdfEllipse(pt, center, abs(offset1));
			float a = antialiase(d);
			return min(a_rect, a);
		}
		center = vec2(extent.x + offset2.x, -extent.y + offset2.y); 
		if (is_top_right(pt, extent, center)) {
			float d = sdfEllipse(pt, center, abs(offset2));
			float a = antialiase(d);
			return min(a_rect, a);
		}
		center = vec2(extent.x + offset3.x, extent.y + offset3.y); 
		if (is_right_bottom(pt, extent, center)) {
			float d = sdfEllipse(pt, center, abs(offset3));
			float a = antialiase(d);
			return min(a_rect, a);
		}
		
		center = vec2(-extent.x + offset4.x, extent.y + offset4.y); 
		if (is_bottom_left(pt, extent, center)) {
			float d = sdfEllipse(pt, center, abs(offset4));
			float a = antialiase(d);
			return min(a_rect, a);
		}
		return a_rect;
	}
	float calc_alpha(vec2 vVertexPosition) {
		vec4 scale = clipSdf[0];
		vec2 pos = scale.zw * vVertexPosition - scale.xy;
		vec4 top = clipSdf[2];
		vec4 bottom = clipSdf[3];
		// 左上角
		vec2 c1 = vec2(max(0.01, top.y), max(0.01, top.x));
		// 右上角
		vec2 c2 = vec2(-max(0.01, top.z), max(0.01, top.w));
		// 右下角
		vec2 c3 = vec2(-max(0.01, bottom.y), -max(0.01, bottom.x));
		// 左下角
		vec2 c4 = vec2(max(0.01, bottom.z), -max(0.01, bottom.w));
		
		vec4 extent = clipSdf[1];
		return antialiase_round_rect(pos, extent.xy, c1, c2, c3, c4);
	}
#endif


#ifdef BORDER
	// 边框
    float antialiaseBorderRect(vec2 pt, vec2 extent, vec4 trbl) {
        float r_big = sdfRect(pt, extent);
        float a_big = antialiase(r_big);
        vec2 center = 0.5 * vec2(trbl.w - trbl.y, trbl.x - trbl.z); 
        extent = extent - 0.5 * vec2(trbl.y + trbl.w, trbl.x + trbl.z);
        float r_small = sdfRect(pt - center, extent);
        float a_small = antialiase(r_small);
        
        return a_big - a_small;
    }

	float antialiase_between(float small_d, float big_d) 
    {
        float anti_big_d = 1.0 * fwidth(big_d);
        float a_big = 1.0 - smoothstep(-anti_big_d, anti_big_d, big_d);
        float anti_small_d = 1.0 * fwidth(small_d);
        float a_small = 1.0 - smoothstep(-anti_small_d, anti_small_d, small_d);
        return a_big - a_small;
    }

	float antialiase_border(vec2 pt, vec2 extent, vec2 offset1, vec2 offset2, vec2 offset3, vec2 offset4, vec4 trbl) {
		vec2 center = vec2(-extent.x + offset1.x, -extent.y + offset1.y); 
        vec2 r = pt - center;
        if (r.x < 0.0 && r.y < 0.0) {
			vec2 big = abs(offset1);
			
			vec2 small = big - trbl.wx;
			float small_d = sdfEllipse(pt, center, small);
			
			float big_d = sdfEllipse(pt, center, big);
			return antialiase_between(small_d, big_d);
		}
		center = vec2(extent.x + offset2.x, -extent.y + offset2.y); 
		r = pt - center;
        if (r.x > 0.0 && r.y < 0.0) {
			vec2 big = abs(offset2);
			vec2 small = big - trbl.yx;
			float small_d = sdfEllipse(pt, center, small);
			
			float big_d = sdfEllipse(pt, center, big);
			return antialiase_between(small_d, big_d);
		}
		center = vec2(extent.x + offset3.x, extent.y + offset3.y); 
		r = pt - center;
        if (r.x > 0.0 && r.y > 0.0) {
			vec2 big = abs(offset3);
			
			vec2 small = big - trbl.yz;
			float small_d = sdfEllipse(pt, center, small);
			
			float big_d = sdfEllipse(pt, center, big);
			return antialiase_between(small_d, big_d);
		}
		
		center = vec2(-extent.x + offset4.x, extent.y + offset4.y); 
		r = pt - center;
        if (r.x < 0.0 && r.y > 0.0) {
			vec2 big = abs(offset4);
			
			vec2 small = big - trbl.wz;
			float small_d = sdfEllipse(pt, center, small);
			
			float big_d = sdfEllipse(pt, center, big);
			return antialiase_between(small_d, big_d);
		}
		return antialiaseBorderRect(pt, extent, trbl);
	}
	float calc_alpha(vec2 vVertexPosition) {
		vec4 scale = clipSdf[0];
		vec2 pos = scale.zw * vVertexPosition - scale.xy;
	
        vec4 top = clipSdf[2];
		vec4 bottom = clipSdf[3];
		// 左上角
		vec2 c1 = vec2(max(0.01, top.y), max(0.01, top.x));
		// 右上角
		vec2 c2 = vec2(-max(0.01, top.z), max(0.01, top.w));
		// 右下角
		vec2 c3 = vec2(-max(0.01, bottom.y), -max(0.01, bottom.x));
		// 左下角
		vec2 c4 = vec2(max(0.01, bottom.z), -max(0.01, bottom.w));
		
		vec4 param1 = clipSdf[1];
		vec2 extent = param1.xy;
		// 上-右-下-左
		vec4 trbl = vec4(param1.zw, textureSizeOrBottomLeftBorder);
		return antialiase_border(pos, extent, c1, c2, c3, c4, trbl);
	}
#endif

#ifdef SHADOW
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
#endif

void main() {
	vec4 c = color.rgba;

	// 顶点色
	#ifdef VERT_COLOR
		c = vColor;
	#endif

	#ifdef SHADOW
		c.a = c.a * getShadowAlpha(vVertexPosition, strokeColorOrURect.xy, strokeColorOrURect.zw, blur / 2.0);
	#endif

	#ifdef BORDER_RADIUS
		c.a = c.a * calc_alpha(vVertexPosition);
	#endif

	#ifdef BORDER
		c.a = c.a * calc_alpha(vVertexPosition);
	#endif

	o_Target = c;
	// o_Target = vec4( getShadowAlpha(vVertexPosition, uRect.xy, uRect.zw, blur / 2.0), 0.0, 0.0, 1.0);
	// o_Target = vec4(color.rgb, 0.5);
	// o_Target = vec4( getShadowAlpha(vVertexPosition, uRect.xy, uRect.zw, blur / 2.0), 0.0, 0.0, 1.0);
	// o_Target = vec4(vVertexPosition.x, vVertexPosition.y, 0.0, 1.0);
	// o_Target = vec4(uRect.x, uRect.y, uRect.z, uRect.w);
}