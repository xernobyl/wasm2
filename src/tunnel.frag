precision highp float;

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

uniform vec2 u_pos;

#define PI_TWO			1.570796326794897
#define PI				3.141592653589793
#define TWO_PI			6.283185307179586


float sdRoundBox(vec3 p, vec3 b, float r)
{
	vec3 d = abs(p) - b;
	return length(max(d, 0.0)) - r + min(max(d.x, max(d.y, d.z)), 0.0);
}


void CamPolar(out vec3 pos, out vec3 ray, in vec3 origin, in vec2 rotation, in float distance, in float zoom)
{
	// get rotation coefficients
	vec2 c = vec2(cos(rotation.x), cos(rotation.y));
	vec4 s;
	s.xy = vec2(sin(rotation.x), sin(rotation.y));
	s.zw = -s.xy;

	// ray in view space
	ray.xy = uv;
    ray.xy *= atan(u_resolution.y, u_resolution.x);
    ray.x *= u_resolution.x / u_resolution.y;
    ray.z = zoom;

	ray = normalize(ray);
	localRay = ray;

	// rotate ray
	ray.yz = ray.yz * c.xx + ray.zy * s.zx;
	ray.xz = ray.xz * c.yy + ray.zx * s.yw;

	// position camera
	pos = origin - distance*vec3(c.x*s.y,s.z,c.x*c.y);
}


void main() {
	float ar = u_resolution.x / u_resolution.y;
	vec2 uv = gl_FragCoord.xy / u_resolution.xy;
	uv = uv * 2.0 - 1.0;
	uv *= vec2(ar / 1.0 / sqrt(ar * ar + 1.0), 1.0 / sqrt(ar * ar + 1.0));

	vec3 dir = normalize(vec3(uv, -1.0));

	vec3 pos = vec3(5.0 * cos(u_time), 0.0, 5.0 * sin(u_time));
	vec3 iPos = pos;
	float d;

	for (int i = 0; i < 64; ++i) {
		d = sdRoundBox(pos, vec3(2.0, 2.0, 2.0), 0.01);
		if (d < 0.001) {
			gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0);
			return;
		}
		pos = iPos + dir * d;
	}

	gl_FragColor = vec4(1.0, 0.0, 1.0, 1.0);
}
