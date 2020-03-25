/*{
    "CATEGORIES": [
        "Stylize"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "LABEL": "H Shift",
            "MAX": 0.05,
            "MIN": -0.05,
            "NAME": "hShift",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "V Shift",
            "MAX": 0.05,
            "MIN": -0.05,
            "NAME": "vShift",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "Shift Mix",
            "MAX": 1,
            "MIN": 0,
            "NAME": "mixAmount1",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "Original Mix",
            "MAX": 1,
            "MIN": 0,
            "NAME": "mixAmount2",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/



void main()
{
	vec2 loc = isf_FragNormCoord;
	vec2 shift = vec2(hShift, vShift);

	//	zoom slightly so that there aren't out of range pixels
	float zoomAmount = 1.0 + 2.0 * max(hShift, vShift);
	vec2 modifiedCenter = vec2(0.5,0.5);
	loc.x = (loc.x - modifiedCenter.x)*(1.0/zoomAmount) + modifiedCenter.x;
	loc.y = (loc.y - modifiedCenter.y)*(1.0/zoomAmount) + modifiedCenter.y;
	
	vec4 color = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord);
	vec4 colorL = IMG_NORM_PIXEL(inputImage, clamp(loc - shift,0.0,1.0));
	vec4 colorR = IMG_NORM_PIXEL(inputImage, clamp(loc + shift,0.0,1.0));
	
	vec4 outColor = mix(min(colorL, colorR), max(colorL, colorR), mixAmount1);
	outColor =  mix(min(outColor, color), max(outColor, color), mixAmount2);
	
	gl_FragColor = outColor;
}