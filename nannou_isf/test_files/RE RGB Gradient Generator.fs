/*{
    "CATEGORIES": [
        "Color",
        "Utility"
    ],
    "CREDIT": "by Carter Rosenberg",
    "INPUTS": [
        {
            "NAME": "lookupImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 1,
            "MAX": 16,
            "MIN": 0.5,
            "NAME": "frequency1",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "phase1",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 2,
            "MIN": -2,
            "NAME": "amplitude1",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": -1,
            "NAME": "offset1",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "angle1",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABELS": [
                "Ramp",
                "Triangle",
                "Sine",
                "Exponential",
                "Look Up Table"
            ],
            "NAME": "curve1",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2,
                3,
                4
            ]
        },
        {
            "DEFAULT": 1,
            "MAX": 1,
            "MIN": -1,
            "NAME": "mixLevel1",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0,
                0,
                0,
                1
            ],
            "NAME": "startColor1",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                1,
                0,
                0,
                1
            ],
            "NAME": "endColor1",
            "TYPE": "color"
        },
        {
            "DEFAULT": 1,
            "MAX": 16,
            "MIN": 0.5,
            "NAME": "frequency2",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "phase2",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 2,
            "MIN": -2,
            "NAME": "amplitude2",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": -1,
            "NAME": "offset2",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.75,
            "MAX": 1,
            "MIN": 0,
            "NAME": "angle2",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABELS": [
                "Ramp",
                "Triangle",
                "Sine",
                "Exponential",
                "Look Up Table"
            ],
            "NAME": "curve2",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2,
                3,
                4
            ]
        },
        {
            "DEFAULT": 1,
            "MAX": 1,
            "MIN": -1,
            "NAME": "mixLevel2",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0,
                0,
                0,
                1
            ],
            "NAME": "startColor2",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0,
                1,
                0,
                1
            ],
            "NAME": "endColor2",
            "TYPE": "color"
        },
        {
            "DEFAULT": 2,
            "MAX": 16,
            "MIN": 0.5,
            "NAME": "frequency3",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "phase3",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 2,
            "MIN": -2,
            "NAME": "amplitude3",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": -1,
            "NAME": "offset3",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "angle3",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABELS": [
                "Ramp",
                "Triangle",
                "Sine",
                "Exponential",
                "Look Up Table"
            ],
            "NAME": "curve3",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2,
                3,
                4
            ]
        },
        {
            "DEFAULT": 1,
            "MAX": 1,
            "MIN": -1,
            "NAME": "mixLevel3",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0,
                0,
                0,
                1
            ],
            "NAME": "startColor3",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0,
                0,
                1,
                1
            ],
            "NAME": "endColor3",
            "TYPE": "color"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "FLOAT": true
        }
    ]
}
*/



const float pi = 3.14159265359;
const float e = 2.71828182846;


float doMath(int curve, float freq, float phase, float val, vec2 pt)	{
	float	returnMe = phase + freq * val;

	if (curve == 0)	{
		returnMe = mod(returnMe,1.0);
	}
	else if (curve == 1)	{
		returnMe = mod(2.0 * returnMe,2.0);
		returnMe = (returnMe < 1.0) ? returnMe : 1.0 - (returnMe - floor(returnMe));
	}
	else if (curve == 2)	{
		returnMe = sin(returnMe * pi * 2.0 - pi / 2.0) * 0.5 + 0.5;
	}
	else if (curve == 3)	{
		returnMe = mod(2.0 * returnMe, 2.0);
		returnMe = (returnMe < 1.0) ? returnMe : 1.0 - (returnMe - floor(returnMe));
		returnMe = pow(returnMe, 2.0);
	}
	else if (curve == 4)	{
		vec2	loc = mod(pt * freq + phase,1.0);
		vec4	tmp = IMG_NORM_PIXEL(lookupImage,loc);
		returnMe = (tmp.r+tmp.g+tmp.b)*tmp.a/3.0;
	}
	return returnMe;	
}

//	note that this works on normalized points, but respects aspect ratio
vec2 rotatePoint(vec2 pt, float angle)	{
	vec2 returnMe = pt * RENDERSIZE;;

	float r = distance(RENDERSIZE/2.0, returnMe);
	float a = atan ((returnMe.y-RENDERSIZE.y/2.0),(returnMe.x-RENDERSIZE.x/2.0));

	returnMe.x = r * cos(a + 2.0 * pi * angle - pi) + 0.5;
	returnMe.y = r * sin(a + 2.0 * pi * angle - pi) + 0.5;
	
	returnMe = returnMe / RENDERSIZE + vec2(0.5);
	
	return returnMe;
}


void main() {
	vec4 returnMe = vec4(0.0);
	vec4 blendColor = vec4(0.0);
	float mixAmount = 0.0;
	vec2 loc = isf_FragNormCoord;

	loc = rotatePoint(isf_FragNormCoord,angle1);
	mixAmount = doMath(curve1,frequency1,phase1,1.0-loc.x,loc);
	mixAmount = (amplitude1 >= 0.0) ? mixAmount * amplitude1 : (1.0 - mixAmount) * abs(amplitude1);
	mixAmount += offset1;
	blendColor = mix(startColor1,endColor1,mixAmount);
	returnMe.rgb += blendColor.rgb * mixLevel1;
	returnMe.a += abs(blendColor.a);
	
	loc = rotatePoint(isf_FragNormCoord,angle2);
	mixAmount = doMath(curve2,frequency2,phase2,1.0-loc.x,loc);
	mixAmount = (amplitude2 >= 0.0) ? mixAmount * amplitude2 : (1.0 - mixAmount) * abs(amplitude2);
	mixAmount += offset2;
	blendColor = mix(startColor2,endColor2,mixAmount);
	returnMe.rgb += blendColor.rgb  * mixLevel2;
	returnMe.a += abs(blendColor.a);
	
	loc = rotatePoint(isf_FragNormCoord,angle3);
	mixAmount = doMath(curve3,frequency3,phase3,1.0-loc.x,loc);
	mixAmount = (amplitude3 >= 0.0) ? mixAmount * amplitude3 : (1.0 - mixAmount) * abs(amplitude3);
	mixAmount += offset3;
	blendColor = mix(startColor3,endColor3,mixAmount);
	returnMe.rgb += blendColor.rgb  * mixLevel3;
	returnMe.a += abs(blendColor.a);
	
	gl_FragColor = returnMe;
}