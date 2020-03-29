/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Blur"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "width",
			"LABEL": "Width",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "angle",
			"LABEL": "Angle",
			"TYPE": "float",
			"MIN": -1.0,
			"MAX": 1.0,
			"DEFAULT": 0.125
		},
		{
			"NAME": "quality",
			"LABEL": "Quality",
			"VALUES": [
				24,
				16,
				8,
				4
			],
			"LABELS": [
				"Low",
				"Mid",
				"High",
				"Best"
			],
			"DEFAULT": 16,
			"TYPE": "long"
		}
	]
}*/


const float pi = 3.14159265359;


void main() {
	vec2 loc = isf_FragNormCoord * RENDERSIZE;

	vec2 p1 = vec2(0.0);
	vec2 p2 = vec2(1.0);
	vec2 vector = vec2(cos(pi * angle),sin(pi * angle));
	
	vec4 returnMe;
	
	if (width > 0.0)	{
		p1 = loc - width * RENDERSIZE * vector;
		p2 = loc + width * RENDERSIZE * vector;
		
		//	now we have the two points to smear between,
		float i;
		float count = clamp(width * max(RENDERSIZE.x,RENDERSIZE.y) / float(quality), 5.0, 75.0);
		vec2 diff = p2 - p1;
		for (float i = 0.0; i < 75.0; ++i)	{
			if (i > float(count))
				break;
			float tmp = (i / (count - 1.0));
			returnMe = returnMe + IMG_PIXEL(inputImage, p1 + diff * tmp) / count;
		}
		
		vector = vec2(cos(pi * (0.5+ angle)),sin(pi * (0.5+ angle)));
		p1 = loc - width * RENDERSIZE * vector;
		p2 = loc + width * RENDERSIZE * vector;
		
		//	now we have the two points to smear between,
		diff = p2 - p1;
		for (float i = 0.0; i < 75.0; ++i)	{
			if (i > float(count))
				break;
			float tmp = (i / (count - 1.0));
			returnMe = returnMe + IMG_PIXEL(inputImage, p1 + diff * tmp) / count;
		}
		returnMe = returnMe / 2.0;
	}
	else	{
		returnMe = IMG_THIS_PIXEL(inputImage);
	}
	gl_FragColor = returnMe;
}