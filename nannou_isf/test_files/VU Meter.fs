/*{
    "CATEGORIES": [
        "Geometry"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "DEFAULT": 1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "audioLevel",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0,
                1,
                0,
                1
            ],
            "NAME": "color1",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                1,
                1,
                0,
                1
            ],
            "NAME": "color2",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                1,
                0,
                0,
                1
            ],
            "NAME": "color3",
            "TYPE": "color"
        }
    ],
    "ISFVSN": "2"
}
*/


const float divisionCount = 5.0;

float round(float val)	{
	if (fract(val) <= 0.5)
		return floor(val);
	else
		return ceil(val);	
}

void main()	{
	vec4		inputPixelColor = vec4(0.0);
	vec2		loc = isf_FragNormCoord;
	
	float		div = floor(divisionCount * audioLevel);
	float		thisDiv = floor(loc.x * divisionCount);
	float		nearestDiv = round(loc.x * divisionCount);
	
	if (loc.x < div / divisionCount)	{
		if (thisDiv <= divisionCount * 0.5)
			inputPixelColor = color1;
		else if (thisDiv <= divisionCount - 2.0)
			inputPixelColor = color2;
		else
			inputPixelColor = color3;
	}
	
	gl_FragColor = inputPixelColor;
}
