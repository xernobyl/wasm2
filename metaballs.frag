precision highp float;

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

uniform vec2 u_pos;

#define PI_TWO			1.570796326794897
#define PI				3.141592653589793
#define TWO_PI			6.283185307179586

vec2 coord(in vec2 p) {
	p = p / u_resolution.xy;
	// correct aspect ratio
	if (u_resolution.x > u_resolution.y) {
		p.x *= u_resolution.x / u_resolution.y;
		p.x += (u_resolution.y - u_resolution.x) / u_resolution.y / 2.0;
	} else {
		p.y *= u_resolution.y / u_resolution.x;
		p.y += (u_resolution.x - u_resolution.y) / u_resolution.x / 2.0;
	}
	// centering
	p -= 0.5;
	p *= vec2(-1.0, 1.0);
	return p;
}


#define _rx (1.0 / min(u_resolution.x, u_resolution.y))
#define _uv (gl_FragCoord.xy / u_resolution.xy)
#define _st coord(gl_FragCoord.xy)
#define _mx coord(u_mouse)


int n_points = 32;
vec3 points[32];

vec2 uv;

float metaballs(vec3 pos)
{
    float METABALLS_TRESHOLD = 1.0;;

    float value = 0.0;
	for (int i = 1; i < 32; ++i) {
		vec3 t = pos - points[i].xyz;
		value += ((cos(u_time) * 0.5 + 0.5) * 0.01 + 0.02) / dot(t, t);
	}

	return METABALLS_TRESHOLD - value;
}

float planes_distance(vec3 pos) {
    return min(1.0 - pos.y, pos.y + 1.0);
}

float opSmoothUnion( float d1, float d2, float k ) {
    float h = clamp( 0.5 + 0.5*(d2-d1)/k, 0.0, 1.0 );
    return mix( d2, d1, h ) - k*h*(1.0-h);
}

float distance_field(vec3 pos) {
    float dm = metaballs(pos);
    float dp = planes_distance(pos);

    return opSmoothUnion(dm, dp, 0.5);
}

vec3 Normal(vec3 pos)
{
	return normalize(pos);

    vec2 d = vec2(0.008 * length(pos),0);

	return normalize(vec3(distance_field(pos + d.xyy) - distance_field(pos - d.xyy),
		distance_field(pos + d.yxy) - distance_field(pos - d.yxy),
		distance_field(pos + d.yyx) - distance_field(pos - d.yyx)));
}

vec3 slerp(vec3 a, vec3 b, float t)
{
	float cosTheta = dot(a, b);
	float theta = acos(cosTheta);
	float sinTheta = sin(theta);

	if (sinTheta <= 0.0000001)
		return b;

	float w1 = sin((1.0 - t) * theta) / sinTheta;
	float w2 = sin(t * theta) / sinTheta;

	return a * w1 + b * w2;
}

vec3 Shade(vec3 pos, vec3 ray, vec3 cpos)
{
	//return abs(fract(pos));

    vec3 norm = Normal(pos);

    float fog = clamp(1.0 - distance(pos, cpos) / 10.0, 0.0, 1.0);
    float light = clamp(dot(normalize(ray), -norm), 0.0, 1.0);

    //return vec3(norm * 0.5 + 0.5);
    return vec3(1.0) * fog * fog * light;
}


float Trace(vec3 pos, vec3 ray)
{
    float h;
	float t = 0.25;

	for (int i = 0; i < 96; ++i)
	{
		vec3 p = pos + t * ray;
        h = distance_field(p);

        if (h <= 0.0 || t > 32.0)
            break;
        t += h;
	}

	if ( t > 32.0 )
		return 0.5;

	return t;
}


vec3 localRay;

// Set up a camera looking at the scene.
// origin - camera is positioned relative to, and looking at, this point
// distance - how far camera is from origin
// rotation - about x & y axes, by left-hand screw rule, relative to camera looking along +z
// zoom - the relative length of the lens
void CamPolar( out vec3 pos, out vec3 ray, in vec3 origin, in vec2 rotation, in float distance, in float zoom )
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
	points[0] = vec3(0.24803918541230538, 0.0, 0.96875);
	points[1] = vec3(-0.31171691542630875, 0.2855582289780974, 0.90625);
	points[2] = vec3(0.0469245666682833, -0.5346812345154763, 0.84375);
	points[3] = vec3(0.37979864779041805, 0.4953800809848635, 0.78125);
	points[4] = vec3(-0.684640374376779, -0.12110324220772836, 0.71875);
	points[5] = vec3(0.6366500979526448, -0.404984679064391, 0.65625);
	points[6] = vec3(-0.20889049343814922, 0.777062223538866, 0.59375);
	points[7] = vec3(-0.3904873942516945, -0.7518597159248004, 0.53125);
	points[8] = vec3(0.8297315040618276, 0.30301661450702244, 0.46875);
	points[9] = vec3(-0.8446318161762448, 0.34865173540772476, 0.40625);
	points[10] = vec3(0.39801732875976853, -0.8505399129417375, 0.34375);
	points[11] = vec3(0.2872031315804752, 0.915648840282326, 0.28125);
	points[12] = vec3(-0.844256605171993, -0.48926395955911317, 0.21875);
	points[13] = vec3(0.9646797918511932, -0.21208214609895487, 0.15625);
	points[14] = vec3(-0.57259642692859, 0.8144594952289597, 0.09375);
	points[15] = vec3(-0.12844792511157987, -0.9912237729365303, 0.03125);
	points[16] = vec3(0.7642755404909269, 0.6441322346438668, -0.03125);
	points[17] = vec3(-0.994745585111356, 0.0411358529930508, -0.09375);
	points[18] = vec3(0.7001232449004646, -0.6967161397944244, -0.15625);
	points[19] = vec3(-0.04507273241261577, 0.9747393940396892, -0.21875);
	points[20] = vec3(-0.6148466213076331, -0.7367917411091062, -0.28125);
	points[21] = vec3(0.9306748320860563, 0.12522098235356116, -0.34375);
	points[22] = vec3(-0.750069130933915, 0.5218785647255897, -0.40625);
	points[23] = vec3(0.19387467028695743, -0.8617923472166155, -0.46875);
	points[24] = vec3(0.4212191481735503, 0.7350835780453462, -0.53125);
	points[25] = vec3(-0.7665838376566645, -0.24456115256430386, -0.59375);
	points[26] = vec3(0.6849679257367517, -0.31647255522697687, -0.65625);
	points[27] = vec3(-0.26841615129105584, 0.6413666714338195, -0.71875);
	points[28] = vec3(-0.21126809336160696, -0.5873791196725938, -0.78125);
	points[29] = vec3(0.4751133708523767, 0.24970627212245172, -0.84375);
	points[30] = vec3(-0.40877899284392705, 0.10775283063337437, -0.90625);
	points[31] = vec3(0.13414895206255345, -0.20863244273247328, -0.96875);

    for (int i = 0; i < 32; ++i) {
        points[i].y = fract(points[i].y * 2.0 - u_time * 0.25) * 2.5 - 1.25;
    }

    uv = gl_FragCoord.xy / u_resolution.xy * vec2(4.0) - vec2(2.0);

    vec2 camRot = vec2(0.0, 0.0);	//(iMouse.yx / u_resolution.yx * vec2(-2.0, 2.0) + 1.0) * vec2(3.1415926535897932384626433832795, 1.5707963267948966192313216916398);

	vec3 pos, ray;
	CamPolar(pos, ray, vec3(0), camRot, 4.0, 2.0);

    float ts0 = Trace(pos, ray);

    vec3 linear;

    if (ts0 > 0.0)
        linear = Shade(pos + ray * ts0, ray, pos);
    else
        linear = vec3(0.0, 1.0, 0.0);

    gl_FragColor = vec4(pow(linear, vec3(1.0 / 2.2)), 1.0);
}
