/*{
	"DESCRIPTION": "demonstrates the use of long-type inputs as pop-up buttons to display either the red, green, or blue channel of an image",
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
			"NAME": "longInputIsPopUpButton",
			"VALUES": [
				0,
				1,
				2
			],
			"LABELS": [
				"red",
				"green",
				"blue"
			],
			"DEFAULT": 1,
			"TYPE": "long"
		}
	]
}*/

void main()
{
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	if (longInputIsPopUpButton == 0)
		gl_FragColor = srcPixel.rrra;
	else if (longInputIsPopUpButton == 1)
		gl_FragColor = srcPixel.ggga;
	else if (longInputIsPopUpButton == 2)
		gl_FragColor = srcPixel.bbba;
}
