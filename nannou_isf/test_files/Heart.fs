/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Geometry"
	],
	"INPUTS": [
		{
			"NAME": "size",
			"LABEL": "Size",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.25
		},
		{
			"NAME": "color",
			"LABEL": "Fill Color",
			"TYPE": "color",
			"DEFAULT": [
				1.0,
				0.0,
				0.0,
				1.0
			]
		}
	]
}*/


//	Adapted from https://glsl.io/transition/d71472a550601b96d69d

 
bool inHeart (vec2 p, vec2 center, float size) {
	if (size == 0.0) return false;
	vec2 o = (p-center)/(1.6*size);
	return pow(o.x*o.x+o.y*o.y-0.3, 3.0) < o.x*o.x*pow(o.y, 3.0);
}
 
void main() {
	vec2 p = isf_FragNormCoord;
	float m = inHeart(p, vec2(0.5, 0.4), size) ? 1.0 : 0.0;
	gl_FragColor = mix(vec4(0.0), color, m);
}