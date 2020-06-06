/*{
	"DESCRIPTION": "demonstrates the use of a 2d point-type input",
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
			"NAME": "location",
			"TYPE": "point2D",
			"DEFAULT": [
				0,
				0
			]
		},
		{
			"NAME": "locationB",
			"TYPE": "point2D",
			"DEFAULT": [
				0,
				0
			],
			"MIN": [
				0,
				0
			],
			"MAX": [
				1920,
				1080
			]
		}
	]
}*/

void main()
{
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	if (abs(gl_FragCoord.xy.x-location.x)<10.0 && abs(gl_FragCoord.xy.y-location.y)<10.0)
		gl_FragColor = vec4(1,1,1,1);
	else
		gl_FragColor = srcPixel;
}
