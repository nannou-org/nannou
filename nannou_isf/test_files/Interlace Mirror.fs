/*{
    "CATEGORIES": [
        "Glitch",
        "Retro"
    ],
    "CREDIT": "by Carter Rosenberg",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Horizontal",
            "NAME": "horizontal",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Vertical",
            "NAME": "vertical",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2"
}
*/

void main()
{
	vec2 pixelCoord = isf_FragNormCoord * RENDERSIZE;
	vec2 loc = pixelCoord;
	if (vertical)	{
		if (mod(pixelCoord.x,2.0)>1.0)	{
			loc.y = RENDERSIZE.y - pixelCoord.y;
		}
	}
	if (horizontal)	{
		if (mod(pixelCoord.y,2.0)>1.0)	{
			loc.x = RENDERSIZE.x - pixelCoord.x;
		}	
	}
	gl_FragColor = IMG_PIXEL(inputImage,loc);
}
