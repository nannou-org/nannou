/*{
	"DESCRIPTION": "demonstrates the use of IMG_THIS_NORM_PIXEL to fetch a pixel color from an input",
	"CREDIT": "by zoidberg",
	"ISFVSN": "2.0",
	"CATEGORIES": [
		"TEST-GLSL FX"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		}
	]
}*/

void main()
{
	vec4		test = IMG_THIS_NORM_PIXEL(inputImage);
	gl_FragColor = test;
}
