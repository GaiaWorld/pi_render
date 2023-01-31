
			use pi_render::rhi::shader::{ShaderMeta, CodeSlice, BlockCodeAtom, InOut, Define,  ShaderVarying, ShaderInput, ShaderOutput, ShaderProgram, merge_defines};
			use render_derive::{Input};
			
			
			
			
				#[derive(Input)]
				#[location(0)]
				pub struct PositionVert;
			

				#[derive(Input)]
				#[location(1)]
				pub struct VertColorVert;
			

			pub struct ProgramMeta;
			impl ShaderProgram for ProgramMeta {
				fn create_meta() -> pi_render::rhi::shader::ShaderMeta {
					let mut meta_ = ShaderMeta::default();
					meta_.name = std::any::type_name::<Self>().to_string();
					let _defines: &[Define] = &[];
					let meta = &mut meta_;
					
					
					
					let visibility = wgpu::ShaderStages::FRAGMENT;

					
					super::sdf::push_meta(meta, visibility, &[]);
					let visibility = wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT;

					
					super::ui_meterial::push_meta(meta, visibility, &[]);
					push_vs_code(&mut meta.vs);
					push_fs_code(&mut meta.fs);
					meta.varyings = ShaderVarying(vec![
						
				InOut::new("vVertexPosition", "vec2", 0, vec![])
			,
				InOut::new("vColor", "vec4", 1, vec![Define::new(true, VERT_COLOR_DEFINE.clone())])
			
					]);
					meta.ins = ShaderInput(vec![
						
				InOut::new("position", "vec2", 0, vec![])
			,
				InOut::new("vertColor", "vec4", 1, vec![Define::new(true, VERT_COLOR_DEFINE.clone())])
			
					]);
					meta.outs = ShaderOutput(vec![
						
				InOut::new("o_Target", "vec4", 0, vec![])
			
					]);
					meta_
				}
			}
			fn push_vs_code(_codes: &mut BlockCodeAtom) {
				let _defines: &[Define] = &[];
				_codes.define.push(VS_CODE[0].clone().push_defines_front(_defines));
super::ui_meterial::push_code(codes, merge_defines(_defines, &[]).as_slice());
_codes.running.push(VS_CODE[1].clone().push_defines_front(_defines));
_codes.running.push(VS_CODE[2].clone().push_defines_front(_defines));
			}

			fn push_fs_code(_codes: &mut BlockCodeAtom) {
				let _defines: &[Define] = &[];
				_codes.define.push(FS_CODE[0].clone().push_defines_front(_defines));
super::ui_meterial::push_code(codes, merge_defines(_defines, &[]).as_slice());
super::sdf::push_code(codes, merge_defines(_defines, &[]).as_slice());
_codes.define.push(FS_CODE[1].clone().push_defines_front(_defines));
_codes.define.push(FS_CODE[2].clone().push_defines_front(_defines));
_codes.define.push(FS_CODE[3].clone().push_defines_front(_defines));
_codes.define.push(FS_CODE[4].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[5].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[6].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[7].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[8].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[9].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[10].clone().push_defines_front(_defines));
			}

			lazy_static! {
				static ref VS_CODE: Vec<CodeSlice> = vec![CodeSlice{code:pi_atom::Atom::from("#version 450
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	vVertexPosition = position;
	gl_Position = project * view * world * vec4(position.x, position.y, 1.0, 1.0);
	gl_Position.z = depth/60000.0;
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("		vColor = vertColor;
"), defines: vec![Define::new(true, VERT_COLOR_DEFINE.clone())]}];
				static ref FS_CODE: Vec<CodeSlice> = vec![CodeSlice{code:pi_atom::Atom::from("#version 450

precision highp float;
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("float sdfEllipse(vec2 pt, vec2 center, vec2 ab)

{

	pt -= center;

	vec2 recAB = 1.0 / ab;

	vec2 scale = pt * recAB;

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

float antialiase(float d) 

{

	float anti = 1.0 * fwidth(d);

	return 1.0 - smoothstep(-anti, anti, d);

}
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	float antialiase_round_rect(vec2 pt, vec2 extent, vec2 offset1, vec2 offset2, vec2 offset3, vec2 offset4) {

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

		vec2 c1 = vec2(max(0.01, top.y), max(0.01, top.x));

		vec2 c2 = vec2(-max(0.01, top.z), max(0.01, top.w));

		vec2 c3 = vec2(-max(0.01, bottom.y), -max(0.01, bottom.x));

		vec2 c4 = vec2(max(0.01, bottom.z), -max(0.01, bottom.w));

		vec4 extent = clipSdf[1];

		return antialiase_round_rect(pos, extent.xy, c1, c2, c3, c4);

	}
"), defines: vec![Define::new(true, BORDER_RADIUS_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("    float antialiaseBorderRect(vec2 pt, vec2 extent, vec4 trbl) {

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

		vec2 c1 = vec2(max(0.01, top.y), max(0.01, top.x));

		vec2 c2 = vec2(-max(0.01, top.z), max(0.01, top.w));

		vec2 c3 = vec2(-max(0.01, bottom.y), -max(0.01, bottom.x));

		vec2 c4 = vec2(max(0.01, bottom.z), -max(0.01, bottom.w));

		vec4 param1 = clipSdf[1];

		vec2 extent = param1.xy;

		vec4 trbl = vec4(param1.zw, textureSizeOrBottomLeftBorder);

		return antialiase_border(pos, extent, c1, c2, c3, c4, trbl);

	}
"), defines: vec![Define::new(true, BORDER_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("	float erf(float x) {

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

	float erfSigma(float x, float sigma) {

		return erf(x / (sigma * 1.4142135623730951));

	}

	float colorFromRect(vec2 p0, vec2 p1, float sigma) {

		return (erfSigma(p1.x, sigma) - erfSigma(p0.x, sigma)) *

			(erfSigma(p1.y, sigma) - erfSigma(p0.y, sigma)) / 4.0;

	}

	float getShadowAlpha(vec2 pos, vec2 ptMin, vec2 ptMax, float sigma) {

		vec2 dMin = pos - ptMin, dMax = pos - ptMax;

		return colorFromRect(dMin, dMax, sigma);

	}
"), defines: vec![Define::new(true, SHADOW_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("	vec4 c = color.rgba;
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("		c = vColor;
"), defines: vec![Define::new(true, VERT_COLOR_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("		c.a = c.a * getShadowAlpha(vVertexPosition, strokeColorOrURect.xy, strokeColorOrURect.zw, blur / 2.0);
"), defines: vec![Define::new(true, SHADOW_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("		c.a = c.a * calc_alpha(vVertexPosition);
"), defines: vec![Define::new(true, BORDER_RADIUS_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("		c.a = c.a * calc_alpha(vVertexPosition);
"), defines: vec![Define::new(true, BORDER_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("	o_Target = c;
"), defines: vec![]}];
				pub static ref BORDER_DEFINE: pi_atom::Atom = pi_atom::Atom::from("BORDER");
pub static ref SHADOW_DEFINE: pi_atom::Atom = pi_atom::Atom::from("SHADOW");
pub static ref VERT_COLOR_DEFINE: pi_atom::Atom = pi_atom::Atom::from("VERT_COLOR");
pub static ref BORDER_RADIUS_DEFINE: pi_atom::Atom = pi_atom::Atom::from("BORDER_RADIUS");
			}
		