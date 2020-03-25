/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Color Effect"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"LABEL": "Red",
			"NAME": "redInput",
			"TYPE": "long",
			"VALUES": [
				0,
				1,
				2,
				3,
				4
			],
			"LABELS": [
				"R",
				"G",
				"B",
				"A",
				"Average"
			],
			"DEFAULT": 0
		},
		{
			"LABEL": "Green",
			"NAME": "greenInput",
			"TYPE": "long",
			"VALUES": [
				0,
				1,
				2,
				3,
				4
			],
			"LABELS": [
				"R",
				"G",
				"B",
				"A",
				"Average"
			],
			"DEFAULT": 1
		},
		{
			"LABEL": "Blue",
			"NAME": "blueInput",
			"TYPE": "long",
			"VALUES": [
				0,
				1,
				2,
				3,
				4
			],
			"LABELS": [
				"R",
				"G",
				"B",
				"A",
				"Average"
			],
			"DEFAULT": 2
		},
		{
			"LABEL": "Alpha",
			"NAME": "alphaInput",
			"TYPE": "long",
			"VALUES": [
				0,
				1,
				2,
				3,
				4
			],
			"LABELS": [
				"R",
				"G",
				"B",
				"A",
				"Average"
			],
			"DEFAULT": 3
		}
	]
}*/

void main() {
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	vec4		outputPixel = srcPixel;
	float		avgVal = (srcPixel.r + srcPixel.g + srcPixel.b) * srcPixel.a / 3.0;
	
	if (redInput == 0)	{
		outputPixel.r = srcPixel.r;
	}
	else if (redInput == 1)	{
		outputPixel.r = srcPixel.g;
	}
	else if (redInput == 2)	{
		outputPixel.r = srcPixel.b;
	}
	else if (redInput == 3)	{
		outputPixel.r = srcPixel.a;
	}
	else if (redInput == 4)	{
		outputPixel.r = avgVal;
	}
	
	if (greenInput == 0)	{
		outputPixel.g = srcPixel.r;
	}
	else if (greenInput == 1)	{
		outputPixel.g = srcPixel.g;
	}
	else if (greenInput == 2)	{
		outputPixel.g = srcPixel.b;
	}
	else if (greenInput == 3)	{
		outputPixel.g = srcPixel.a;
	}
	else if (greenInput == 4)	{
		outputPixel.g = avgVal;
	}
	
	if (blueInput == 0)	{
		outputPixel.b = srcPixel.r;
	}
	else if (blueInput == 1)	{
		outputPixel.b = srcPixel.g;
	}
	else if (blueInput == 2)	{
		outputPixel.b = srcPixel.b;
	}
	else if (blueInput == 3)	{
		outputPixel.b = srcPixel.a;
	}
	else if (blueInput == 4)	{
		outputPixel.b = avgVal;
	}
	
	
	if (alphaInput == 0)	{
		outputPixel.a = srcPixel.r;
	}
	else if (alphaInput == 1)	{
		outputPixel.a = srcPixel.g;
	}
	else if (alphaInput == 2)	{
		outputPixel.a = srcPixel.b;
	}
	else if (alphaInput == 3)	{
		outputPixel.a = srcPixel.a;
	}
	else if (alphaInput == 4)	{
		outputPixel.a = avgVal;
	}
	
	gl_FragColor = outputPixel;
}