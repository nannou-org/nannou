/*{
    "CATEGORIES": [
        "Kaleidoscope"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "Repllcates a radial slice of an image",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Post Rotate Angle",
            "MAX": 360,
            "MIN": 0,
            "NAME": "postRotateAngle",
            "TYPE": "float"
        },
        {
            "DEFAULT": 12,
            "LABEL": "Number Of Divisions",
            "MAX": 360,
            "MIN": 1,
            "NAME": "numberOfDivisions",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Pre Rotate Angle",
            "MAX": 180,
            "MIN": -180,
            "NAME": "preRotateAngle",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "IDENTITY": 0,
            "LABEL": "Radius Start",
            "MAX": 1,
            "MIN": 0,
            "NAME": "centerRadiusStart",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "IDENTITY": 0,
            "LABEL": "Radius End",
            "MAX": 2,
            "MIN": 0,
            "NAME": "centerRadiusEnd",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/



const float pi = 3.14159265359;




void main()	{
	
	vec4		inputPixelColor = vec4(0.0);
	vec2		loc = _inputImage_imgRect.zw * vec2(isf_FragNormCoord.x,isf_FragNormCoord.y);
	//	'r' is the radius- the distance in pixels from 'loc' to the center of the rendering space
	//float		r = distance(IMG_SIZE(inputImage)/2.0, loc);
	float		r = distance(_inputImage_imgRect.zw/2.0, loc);
	//	'a' is the angle of the line segment from the center to loc is rotated
	//float		a = atan ((loc.y-IMG_SIZE(inputImage).y/2.0),(loc.x-IMG_SIZE(inputImage).x/2.0));
	float		a = atan ((loc.y-_inputImage_imgRect.w/2.0),(loc.x-_inputImage_imgRect.z/2.0));
	float		modAngle = 2.0 * pi / numberOfDivisions;
	float		scaledCenterRadiusStart = centerRadiusStart * max(RENDERSIZE.x,RENDERSIZE.y);
	float		scaledCenterRadiusEnd = centerRadiusEnd * max(RENDERSIZE.x,RENDERSIZE.y);
	
	if (scaledCenterRadiusStart > scaledCenterRadiusEnd)	{
		scaledCenterRadiusStart = scaledCenterRadiusEnd;
		scaledCenterRadiusEnd = centerRadiusStart * max(RENDERSIZE.x,RENDERSIZE.y);
	}
	
	if ((centerRadiusEnd != centerRadiusStart)&&(r >= scaledCenterRadiusStart)&&(r <= scaledCenterRadiusEnd))	{
		r = (r - scaledCenterRadiusStart) / (centerRadiusEnd - centerRadiusStart);
		
		a = mod(a + pi * postRotateAngle/360.0,modAngle);
	
		//	now modify 'a', and convert the modified polar coords (radius/angle) back to cartesian coords (x/y pixels)
		loc.x = r * cos(a + 2.0 * pi * (preRotateAngle) / 360.0);
		loc.y = r * sin(a + 2.0 * pi * (preRotateAngle) / 360.0);
	
		loc = loc / _inputImage_imgRect.zw + vec2(0.5);
	
		if ((loc.x < 0.0)||(loc.y < 0.0)||(loc.x > 1.0)||(loc.y > 1.0))	{
			inputPixelColor = vec4(0.0);
		}
		else	{
			inputPixelColor = IMG_NORM_PIXEL(inputImage,loc);
		}
	}
	gl_FragColor = inputPixelColor;
}
