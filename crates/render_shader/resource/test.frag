
// in vec3 v_normal;
// in vec3 v_pos;

// out vec4 gl_FragColor;

// uniform vec4 emissive


vec4 baseColor = vec4(1., 1., 1., 1.);

baseColor.rgb *= emissive.rgb * emissive.a;

float alpha = 1.0;

// float level = dot(v_normal, vec3(0., 0., -1.));
baseColor.rgb = mix(baseColor.rgb, v_normal, 0.5);
// baseColor.rgb = (v_pos + vec3(1., 1., 1.)) / 2.;

gl_FragColor = vec4(baseColor.rgb, alpha);