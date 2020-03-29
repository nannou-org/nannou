/*{
	"DESCRIPTION": "",
	"CREDIT": "",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Geometry Adjustment", "Utility"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "offset",
			"TYPE": "point2D",
			"DEFAULT": [
				0.5,
				0.5
			],
			"MIN": [
				0.0,
				0.0
			],
			"MAX": [
				1.0,
				1.0
			]
		},
		{
			"NAME": "repeatImage",
			"TYPE": "bool",
			"DEFAULT": 0.0
		}
	]
	
}*/

void main()	{
	vec4		outputColor = vec4(0.0);
	vec2		newLoc = offset;
	vec2		topSize = RENDERSIZE;
	
	newLoc = offset * RENDERSIZE;
	newLoc.x = topSize.x - newLoc.x;
	newLoc.y = topSize.y - newLoc.y;
	newLoc = (gl_FragCoord.xy + 2.0*newLoc) - topSize;
	
	if (repeatImage)	{
		newLoc = mod(newLoc, RENDERSIZE);	
	}
	
	if ((newLoc.x >= 0.0)&&(newLoc.x < RENDERSIZE.x)&&(newLoc.y >= 0.0)&&(newLoc.y <= RENDERSIZE.y))	{
		outputColor = IMG_PIXEL(inputImage, newLoc);
	}
	
	gl_FragColor = outputColor;
}
