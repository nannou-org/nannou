/*{
	"DESCRIPTION": "demonstrates a BOOL-type input on an image filter",
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
			"NAME": "flashToggle",
			"TYPE": "bool",
			"DEFAULT": 1.0
		}
	]
}*/

void main()
{
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	gl_FragColor = (flashToggle==true) ? vec4(1.0,1.0,1.0,1.0) : srcPixel;
}
