/*{
	"DESCRIPTION": "demonstrates the use of float-type inputs",
	"CREDIT": "by zoidberg",
	"ISFVSN": "2.0",
	"CATEGORIES": [
		"TEST-GLSL FX"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "level",
			"TYPE": "float",
			"DEFAULT": 0.5,
			"MIN": 0.0,
			"MAX": 1.0
		}
	]
}*/

void main()
{
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	float		luma = (srcPixel.r+srcPixel.g+srcPixel.b)/3.0;
	vec4		dstPixel = (luma>level) ? srcPixel : vec4(0,0,0,1);
	gl_FragColor = dstPixel;
}
