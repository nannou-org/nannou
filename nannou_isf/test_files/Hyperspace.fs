/*{
    "CATEGORIES": [
        "Distortion Effect"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "centerX",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 2,
            "MIN": 0,
            "NAME": "scrollAmount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "rightScrollOffset",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "IDENTITY": 1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "midHeight",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "NAME": "seamless",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2"
}
*/

void main()	{
	vec2		loc = isf_FragNormCoord.xy;
	if (centerX == 0.0)	{
		
	}
	else if (centerX == 1.0)	{
		
	}
	else if (loc.x < centerX)	{
		loc.x = loc.x / centerX;
		loc.y = mix(0.5 + (loc.y-0.5) / midHeight,loc.y,1.0-loc.x);
		loc.x = loc.x+scrollAmount;
		if (seamless)	{
			loc.x = ((loc.x <= 1.0)||(loc.x >= 2.0)) ? loc.x : 3.0 - loc.x;
		}
		loc.x = mod(loc.x,1.0);
	}
	else	{
		float		rightScroll = mod(rightScrollOffset+scrollAmount,2.0);
		loc.x = (1.0-loc.x) / (1.0-centerX);
		loc.y = mix(0.5 + (loc.y-0.5) / midHeight,loc.y,1.0-loc.x);
		loc.x = loc.x+rightScroll;
		if (seamless)	{
			loc.x = ((loc.x <= 1.0)||(loc.x >= 2.0)) ? loc.x : 3.0 - loc.x;
		}
		loc.x = mod(loc.x,1.0);
	}
	
	vec4		inputPixelColor = vec4(0.0);
	
	if ((loc.y >= 0.0)&&(loc.y <= 1.0))
		inputPixelColor = IMG_NORM_PIXEL(inputImage,loc);
	
	gl_FragColor = inputPixelColor;
}
