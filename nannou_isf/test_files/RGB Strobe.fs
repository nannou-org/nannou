/*{
	"DESCRIPTION": "",
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
			"NAME": "r",
			"TYPE": "bool",
			"DEFAULT": 1.0
		},
		{
			"NAME": "g",
			"TYPE": "bool",
			"DEFAULT": 1.0
		},
		{
			"NAME": "b",
			"TYPE": "bool",
			"DEFAULT": 1.0
		},
		{
			"NAME": "a",
			"TYPE": "bool",
			"DEFAULT": 0.0
		},
		{
			"NAME": "strobeRates",
			"LABEL": "Strobe Rates",
			"TYPE": "color",
			"DEFAULT": [
				0.0,
				0.0,
				0.0,
				0.0
			]
		}
	],
	"PASSES": [
		{
			"TARGET":"lastState",
			"WIDTH": "1",
			"HEIGHT": "1",
			"PERSISTENT": true,
			"DESCRIPTION": "Stores the current strobe state of each of the color channels."
		},
		{
			
		}
	]
	
}*/



void main()
{
	//	if this is the first pass, i'm going to read the position from the "lastPosition" image, and write a new position based on this and the hold variables
	if (PASSINDEX == 0)	{
		vec4		srcPixel = IMG_PIXEL(lastState,vec2(0.5));
		//	i'm only using the X, which is the last render time we reset
		if (strobeRates.r == 0.0)	{
			srcPixel.r = (srcPixel.r == 0.0) ? 1.0 : 0.0;
		}
		else	{
			srcPixel.r = (mod(TIME, strobeRates.r) <= strobeRates.r / 2.0) ? 1.0 : 0.0;
		}
		if (strobeRates.g == 0.0)	{
			srcPixel.g = (srcPixel.g == 0.0) ? 1.0 : 0.0;
		}
		else	{
			srcPixel.g = (mod(TIME, strobeRates.g) <= strobeRates.g / 2.0) ? 1.0 : 0.0;
		}
		if (strobeRates.b == 0.0)	{
			srcPixel.b = (srcPixel.b == 0.0) ? 1.0 : 0.0;
		}
		else	{
			srcPixel.b = (mod(TIME, strobeRates.b) <= strobeRates.b / 2.0) ? 1.0 : 0.0;
		}
		if (strobeRates.a == 0.0)	{
			srcPixel.a = (srcPixel.a == 0.0) ? 1.0 : 0.0;
		}
		else	{
			srcPixel.a = (mod(TIME, strobeRates.a) <= strobeRates.a / 2.0) ? 1.0 : 0.0;
		}
		gl_FragColor = srcPixel;
	}
	//	else this isn't the first pass- read the position value from the buffer which stores it
	else	{
		vec4 lastStateVector = IMG_PIXEL(lastState,vec2(0.5));
		vec4 srcPixel = IMG_THIS_PIXEL(inputImage);
		float red = (r == true) ? 1.0 : 0.0;
		float green = (g == true) ? 1.0 : 0.0;
		float blue = (b == true) ? 1.0 : 0.0;
		float alpha = (a == true) ? 1.0 : 0.0;
		srcPixel.r = (lastStateVector.r == 0.0) ? srcPixel.r : abs(red-srcPixel.r);
		srcPixel.g = (lastStateVector.g == 0.0) ? srcPixel.g : abs(green-srcPixel.g);
		srcPixel.b = (lastStateVector.b == 0.0) ? srcPixel.b : abs(blue-srcPixel.b);
		srcPixel.a = (lastStateVector.a == 0.0) ? srcPixel.a : abs(alpha-srcPixel.a);
		gl_FragColor = srcPixel;
	}
}
