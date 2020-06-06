/*{
	"DESCRIPTION": "demonstrates the use of an event-type input",
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
			"NAME": "flashEvent",
			"TYPE": "event"
		}
	]
}*/

void main()
{
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	gl_FragColor = (flashEvent==true) ? vec4(1.0,1.0,1.0,1.0) : srcPixel;
}
