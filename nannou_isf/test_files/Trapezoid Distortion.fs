/*{
    "CATEGORIES": [
        "Distortion Effect"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "Warps the video into a trapezoid shape",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Top Width",
            "MAX": 1,
            "MIN": 0,
            "NAME": "topWidth",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Bottom Width",
            "MAX": 1,
            "MIN": 0,
            "NAME": "bottomWidth",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Height",
            "MAX": 1,
            "MIN": 0,
            "NAME": "heightScale",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

void main()	{
	vec4		inputPixelColor = vec4(0.0);
	vec2		loc = isf_FragNormCoord.xy;
	if (heightScale > 0.0)	{
		float	heightDivisor = 1.0 / heightScale;
		loc.y = loc.y * heightDivisor + (1.0 - heightDivisor) / 2.0;
		float		currentLineWidth = mix(bottomWidth,topWidth,loc.y);
		if (currentLineWidth > 0.0)	{
			float		lwDivisor = 1.0 / currentLineWidth;
			loc.x = loc.x * lwDivisor + (1.0 - lwDivisor) / 2.0;
	
			if ((loc.x >= 0.0)&&(loc.x <= 1.0)&&(loc.y >= 0.0)&&(loc.y <= 1.0))	{
				inputPixelColor = IMG_NORM_PIXEL(inputImage,loc);	
			}
		}
	}
	gl_FragColor = inputPixelColor;
}
