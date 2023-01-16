// a_position > POSITION
// a_normal > NORMAL

// out v_normal
// out v_pos

mat4 finalWorld = PI_ObjectToWorld;

vec4 position =  vec4(a_position, 1.);
vec4 worldPos =  finalWorld * position;
// vec4 worldPos =  position;

gl_Position = PI_MATRIX_VP * worldPos;
// gl_Position = position;

v_pos = worldPos.xyz;

mat3 normalWorld = mat3(finalWorld);
v_normal = a_normal; // normalize(vec3(finalWorld * vec4(a_normal, 1.0)));