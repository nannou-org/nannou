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
			"NAME": "gamma",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.5
		}
	]
}*/



void main() {
	//	the input gamma range is 0.0-1.0 (normalized).  the actual gamma range i want to use is 0.0 - 5.0.
	//	however, actual gamma 0.0-1.0 is just as interesting as actual gamma 1.0-5.0, so we scale the normalized input to match...
	float		realGamma = (gamma<=0.5) ? (gamma * 2.0) : (((gamma-0.5) * 2.0 * 4.0) + 1.0);
	vec4		tmpColorA = IMG_THIS_PIXEL(inputImage);
	vec4		tmpColorB;
	tmpColorB.rgb = pow(tmpColorA.rgb, vec3(1.0/realGamma));
	tmpColorB.a = tmpColorA.a;
	gl_FragColor = tmpColorB;
}
