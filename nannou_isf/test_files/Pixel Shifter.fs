/*{
    "CATEGORIES": [
        "Distortion Effect"
    ],
    "CREDIT": "by VIDVOX",
    "DESCRIPTION": "Shifts pixels up and down",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Horizontal Phase",
            "MAX": 1,
            "MIN": 0,
            "NAME": "hPhase",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Horizontal Frequency",
            "MAX": 16,
            "MIN": -16,
            "NAME": "hFrequency",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Horizontal Random",
            "MAX": 1,
            "MIN": 0,
            "NAME": "hRandom",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Vertical Phase",
            "MAX": 1,
            "MIN": 0,
            "NAME": "vPhase",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Vertical Frequency",
            "MAX": 16,
            "MIN": -16,
            "NAME": "vFrequency",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Vertical Random",
            "MAX": 1,
            "MIN": 0,
            "NAME": "vRandom",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Sinusoidal",
            "NAME": "doSin",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Mirror",
            "NAME": "mirror",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2"
}
*/


float		PI_CONST = 3.14159265359;


float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}


void main()
{
	//	start with the pixel
	vec2 loc = isf_FragNormCoord;
	float modVal = 1.0;
	
	if (mirror)
		modVal = 2.0;
	
	//	shift the loc.x by the frequency * loc.y + phase
	if (doSin)
		loc.x = mod(hRandom * rand(vec2(TIME * 0.127, loc.x)) + loc.x + sign(hFrequency) * 0.5 * (1.0+cos(2.0 * PI_CONST * (hPhase + hFrequency * loc.y))), modVal);
	else
		loc.x = mod(hRandom * rand(vec2(TIME * 0.129, loc.x)) + loc.x + hFrequency * loc.y + hPhase, modVal);

	//	shift the loc.y by the frequency * loc.x + phase
	if (doSin)
		loc.y = mod(vRandom * rand(vec2(TIME * 0.273, loc.y)) + loc.y + sign(vFrequency) * 0.5 * (1.0+cos(2.0 * PI_CONST * (vPhase + vFrequency * loc.x))), modVal);
	else
		loc.y = mod(vRandom * rand(vec2(TIME * 0.341, loc.y)) +loc.y + vFrequency * loc.x + vPhase, modVal);
		
	if (loc.x > 1.0)
		loc.x = 2.0 - loc.x;
		
	if (loc.y > 1.0)
		loc.y = 2.0 - loc.y;
	
	gl_FragColor = IMG_NORM_PIXEL(inputImage,loc);
}
