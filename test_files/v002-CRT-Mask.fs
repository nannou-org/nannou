/*{
    "CATEGORIES": [
        "Retro",
        "v002"
    ],
    "CREDIT": "by vade",
    "DESCRIPTION": "CRT Mask, emulating the look of curved CRT Displays",
    "IMPORTED": {
        "CRTMask0": {
            "PATH": "v002-CRT-Mask-RGB-Staggered.png"
        },
        "CRTMask1": {
            "PATH": "v002-CRT-Mask-RGB-Shadow.png"
        },
        "CRTMask2": {
            "PATH": "v002-CRT-Mask-RGB-Straight.png"
        },
        "CRTMask3": {
            "PATH": "v002-CRT-Mask-Scanline-Staggered.png"
        }
    },
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "Amount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Style",
            "LABELS": [
                "Staggered",
                "Shadow",
                "Straight",
                "Scanline Staggered"
            ],
            "NAME": "style",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2,
                3
            ]
        }
    ],
    "ISFVSN": "2"
}
*/

void main (void) 
{ 
	vec2 crtcoord = mod(gl_FragCoord.xy, 12.0);
	vec4 crt;
	
	if (style == 0)
		crt = IMG_PIXEL(CRTMask0, crtcoord);
	else if (style == 1)
		crt = IMG_PIXEL(CRTMask1, crtcoord);
	else if (style == 2)
		crt = IMG_PIXEL(CRTMask2, crtcoord);
	else if (style == 3)
		crt = IMG_PIXEL(CRTMask3, crtcoord);

	vec4 image = IMG_THIS_PIXEL(inputImage);
	vec4 result = mix(image, image * crt, Amount);
	
	result.a = image.a;
	
	gl_FragColor = result;
}