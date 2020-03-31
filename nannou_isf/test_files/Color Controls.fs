/*{
	"CREDIT": "by zoidberg",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Color Adjustment", "Utility"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "bright",
			"TYPE": "float",
			"MIN": -1.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "contrast",
			"TYPE": "float",
			"MIN": -4.0,
			"MAX": 4.0,
			"DEFAULT": 1.0
		},
		{
			"NAME": "hue",
			"TYPE": "float",
			"MIN": -1.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "saturation",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 4.0,
			"DEFAULT": 1.0
		}
	]
}*/


vec3 rgb2hsv(vec3 c);
vec3 hsv2rgb(vec3 c);

void main() {
	vec4		tmpColorA = IMG_THIS_PIXEL(inputImage);
	vec4		tmpColorB;
	//	bright
	tmpColorB = tmpColorA + vec4(bright, bright, bright, 0.0);
	//	contrast
	tmpColorA.rgb = ((vec3(2.0) * (tmpColorB.rgb - vec3(0.5))) * vec3(contrast) / vec3(2.0)) + vec3(0.5);
	tmpColorA.a = ((2.0 * (tmpColorB.a - 0.5)) * abs(contrast) / 2.0) + 0.5;
	
	//	convert RGB to HSV
	tmpColorB.xyz = rgb2hsv(clamp(tmpColorA.rgb, 0.0, 1.0));
	tmpColorB.a = tmpColorA.a;
	
	
	//	hue
	tmpColorB.x = mod((tmpColorB.x + hue), 1.0);
	//	saturation
	tmpColorB.y = tmpColorB.y * saturation;
	
	
	//	convert HSV back to RGB
	tmpColorA.rgb = hsv2rgb(clamp(tmpColorB.xyz, 0.0, 1.0));
	tmpColorA.a = tmpColorB.a;
	
	
	gl_FragColor = clamp(tmpColorA, 0.0, 1.0);
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
