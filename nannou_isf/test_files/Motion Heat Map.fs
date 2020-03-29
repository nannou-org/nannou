/*{
	"DESCRIPTION": "Shows where there is motion between frames",
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Color Effect", "Utility"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "motionThreshold",
			"LABEL": "Threshold",
			"TYPE": "float",
			"MAX": 1.0,
			"MIN": 0.0,
			"DEFAULT": 0.1
		},
		{
			"NAME": "motionGain",
			"LABEL": "Gain",
			"TYPE": "float",
			"MAX": 2.0,
			"MIN": 0.0,
			"DEFAULT": 1.0
		},
		{
			"NAME": "motionColor",
			"LABEL": "Motion Color",
			"TYPE": "bool",
			"DEFAULT": 1.0
		},
		{
			"NAME": "displayMode",
			"LABEL": "Display Mode",
			"TYPE": "long",
			"VALUES": [
				0,
				1
			],
			"LABELS": [
				"Motion Only",
				"Motion Overlay"
			],
			"DEFAULT": 1
		}
	],
	"PASSES": [
		{
			"TARGET":"differenceBuffer",
			"FLOAT": true
		},		{
			"TARGET":"lastFrameBuffer",
			"PERSISTENT": true
		},
		{
		
		}
	]
	
}*/


vec4 heatColor(float lum)	{
	vec4 returnMe = vec4(1.0);
	vec4 colors[9];
	//	8 color stages with variable ranges to get this to look right
	//	black, purple, blue, cyan, green, yellow, orange, red, red
	colors[0] = vec4(0.0,0.0,0.0,1.0);
	colors[1] = vec4(0.272,0.0,0.4,1.0);	//	dark deep purple, (RGB: 139, 0, 204)
	colors[2] = vec4(0.0,0.0,1.0,1.0);		//	full blue
	colors[3] = vec4(0.0,1.0,1.0,1.0);		//	cyan
	colors[4] = vec4(0.0,1.0,0.0,1.0);		//	green
	colors[5] = vec4(0.0,1.0,0.0,1.0);		//	green
	colors[6] = vec4(1.0,1.0,0.0,1.0);		//	yellow
	colors[7] = vec4(1.0,0.5,0.0,1.0);		//	orange
	colors[8] = vec4(1.0,0.0,0.0,1.0);		//	red

	int ix = 0;
	float range = 1.0 / 8.0;
	
	//	orange to red
	if (lum > range * 7.0)	{
		ix = 7;
	}
	//	yellow to orange
	else if (lum > range * 6.0)	{
		ix = 6;
	}
	//	green to yellow
	else if (lum > range * 5.0)	{
		ix = 5;
	}
	//	green to green
	else if (lum > range * 4.0)	{
		ix = 4;
	}
	//	cyan to green
	else if (lum > range * 3.0)	{
		ix = 3;
	}
	//	blue to cyan
	else if (lum > range * 2.0)	{
		ix = 2;
	}
	// purple to blue
	else if (lum > range)	{
		ix = 1;
	}
	
	returnMe = colors[ix];

	return returnMe;
}


void main()
{
	vec4		freshPixel = IMG_PIXEL(inputImage,gl_FragCoord.xy);
	vec4		returnMe = freshPixel;
	
	if (PASSINDEX == 0)	{
		vec4		stalePixel = IMG_PIXEL(lastFrameBuffer,gl_FragCoord.xy);
		vec3		rgbChange = abs(freshPixel.rgb - stalePixel.rgb);
		float		totalChange = (rgbChange.r + rgbChange.g + rgbChange.b) / 3.0;
		if (motionColor)
			returnMe = heatColor(totalChange * motionGain);
		else
			returnMe = vec4(totalChange * motionGain);
	}
	else if (PASSINDEX == 1)	{
		
	}
	else	{
		vec4		differencePixel = IMG_PIXEL(differenceBuffer,gl_FragCoord.xy);
		if (displayMode == 0)	{
			returnMe = differencePixel;
		}
		else if (displayMode == 1)	{
			const vec4 	lumacoeff = vec4(0.2126, 0.7152, 0.0722, 0.0);
			float		luma = dot(differencePixel, lumacoeff);
			if (luma > motionThreshold)	{
				returnMe = differencePixel;
			}
		}
	}
	
	gl_FragColor = returnMe;
}
