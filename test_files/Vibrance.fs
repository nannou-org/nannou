/*{
	"CREDIT": "by zoidberg",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Color Adjustment"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "vibrance",
			"TYPE": "float",
			"MIN": -0.25,
			"MAX": 0.6,
			"DEFAULT": 0.0
		}
	]
}*/


vec3 rgb2hsv(vec3 c);
vec3 hsv2rgb(vec3 c);

void main() {
	vec4		tmpColorA = IMG_THIS_PIXEL(inputImage);
	vec3		tmpColorB = rgb2hsv(tmpColorA.rgb);
	float		maxDelta = sqrt(tmpColorB.y) - tmpColorB.y;
	tmpColorB.y = (maxDelta * vibrance) + tmpColorB.y;
	tmpColorA.rgb = hsv2rgb(tmpColorB.rgb);
	gl_FragColor = tmpColorA;
}




vec3 rgb2hsv(vec3 c)	{
	vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	//vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
	//vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
	vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
	vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);
	
	float d = q.x - min(q.w, q.y);
	float e = 1.0e-10;
	return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)	{
	vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
	vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
	return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}