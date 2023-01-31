
				use pi_render::rhi::shader::{ ShaderMeta, CodeSlice, BlockCodeAtom, Define, };
				use render_derive::{};
				
				pub fn push_meta(_meta: &mut ShaderMeta, _visibility: wgpu::ShaderStages, _defines: &[Define]) {
					
					
				}

				pub fn push_code(_codes: &mut BlockCodeAtom, _defines: &[Define]) {
					_codes.define.push(CODE[0].clone().push_defines_front(_defines));
_codes.define.push(CODE[1].clone().push_defines_front(_defines));
_codes.define.push(CODE[2].clone().push_defines_front(_defines));
_codes.define.push(CODE[3].clone().push_defines_front(_defines));
_codes.define.push(CODE[4].clone().push_defines_front(_defines));
_codes.define.push(CODE[5].clone().push_defines_front(_defines));
				}

				lazy_static! {
					static ref CODE: Vec<CodeSlice> = vec![CodeSlice{code:pi_atom::Atom::from("float antialiase(float d) 

{

	float anti = fwidth(d);

	return 1.0 - smoothstep(-anti, anti, d);

}
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	float sdfEllipse(vec2 pt, vec2 center, vec2 ab)

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

		vec2 c1 = vec2(max(0.01, top.y), max(0.01, top.x));

		vec2 c2 = vec2(-max(0.01, top.z), max(0.01, top.w));

		vec2 c3 = vec2(-max(0.01, bottom.y), -max(0.01, bottom.x));

		vec2 c4 = vec2(max(0.01, bottom.z), -max(0.01, bottom.w));

		vec4 extent = clipSdf[1];

		return antialiase_round_rect(pos, extent.xy, c1, c2, c3, c4);

	}
"), defines: vec![Define::new(true, BORDER_RADIUS_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("    float sdfPie(vec2 p, vec2 sc, float r)

    {

        p.x = abs(p.x);

        float d1 = length(p) - r;

        float m = length(p - sc * clamp(dot(p, sc), 0.0, r) );

        float d2 = m * sign(sc.y * p.x - sc.x * p.y);

        return max(d1, d2);

    }

	float calc_alpha(float d) 

    {

        vec4 scale = clipSdf[0];

        vec4 pie2 = clipSdf[1];

        vec4 pie3 = clipSdf[2];

        vec2 axisSC = pie2.xy;

        vec2 sc = pie2.zw;

        float r = pie3.x;

        vec2 pos = scale.zw * vVertexPosition - scale.xy;

        pos = vec2(axisSC.y * pos.x - axisSC.x * pos.y, axisSC.x * pos.x + axisSC.y * pos.y);

        float d = sdfPie(pos, sc, r);

        return antialiase(d);

    }
"), defines: vec![Define::new(true, SECTOR_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("    float sdfRect(vec2 xy, vec2 wh)

    {

        vec2 d = abs(xy) - wh;

        return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);

    }

	float calc_alpha(float d) {

		vec4 scale = clipSdf[0];

		vec4 uExtent = clipSdf[1];

		vec2 pos = scale.zw * vVertexPosition - scale.xy;

		float d = sdfRect(pos, uExtent.xy);

	}
"), defines: vec![Define::new(true, RECT_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("    float sdfEllipseSimple(vec2 xy, vec2 ab)

    {

        vec2 recAB = 1.0 / ab;

        vec2 scale = xy * recAB;

        return dot(scale, scale) - 1.0;

    }

	float calc_alpha(float d) {

		vec4 scale = clipSdf[0];

		vec4 uEllipseAB = clipSdf[1];

		vec2 pos = scale.zw * vVertexPosition - scale.xy;

        float d = sdfEllipseSimple(pos, uEllipseAB.xy);

        return antialiase(d);

	}
"), defines: vec![Define::new(true, ELLIPSE_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("    float sdfCircle(vec2 xy, float r) {

        return length(xy) - r;

    }

	float calc_alpha(float d) {

		vec4 scale = clipSdf[0];

		vec4 radius = clipSdf[1];

		vec2 pos = scale.zw * vVertexPosition - scale.xy;

        float d = sdfCircle(pos, radius.x);

        return antialiase(d);

	}
"), defines: vec![Define::new(true, CIRCLE_DEFINE.clone())]}];
					pub static ref ELLIPSE_DEFINE: pi_atom::Atom = pi_atom::Atom::from("ELLIPSE");
pub static ref CIRCLE_DEFINE: pi_atom::Atom = pi_atom::Atom::from("CIRCLE");
pub static ref SECTOR_DEFINE: pi_atom::Atom = pi_atom::Atom::from("SECTOR");
pub static ref RECT_DEFINE: pi_atom::Atom = pi_atom::Atom::from("RECT");
pub static ref BORDER_RADIUS_DEFINE: pi_atom::Atom = pi_atom::Atom::from("BORDER_RADIUS");
				}
			