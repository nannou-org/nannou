/*{
    "CATEGORIES": [
        "Masking",
        "Geometry Adjustment"
    ],
    "CREDIT": "",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": [
                0,
                0
            ],
            "LABEL": "Bottom left",
            "MAX": [
                1,
                1
            ],
            "MIN": [
                0,
                0
            ],
            "NAME": "pt1",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": [
                1,
                0
            ],
            "LABEL": "Top left",
            "MAX": [
                1,
                1
            ],
            "MIN": [
                0,
                0
            ],
            "NAME": "pt2",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": [
                1,
                1
            ],
            "LABEL": "Top right",
            "MAX": [
                1,
                1
            ],
            "MIN": [
                0,
                0
            ],
            "NAME": "pt3",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": [
                0,
                1
            ],
            "LABEL": "Bottom right",
            "MAX": [
                1,
                1
            ],
            "MIN": [
                0,
                0
            ],
            "NAME": "pt4",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Invert Mask",
            "NAME": "invertMask",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Apply Mask",
            "LABELS": [
                "Apply Mask",
                "Set Alpha",
                "Show Mask"
            ],
            "NAME": "maskApplyMode",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2
            ]
        }
    ],
    "ISFVSN": "2"
}
*/


float sign(vec2 p1, vec2 p2, vec2 p3)
{
	return (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y);
}

bool PointInTriangle(vec2 pt, vec2 v1, vec2 v2, vec2 v3)
{
	bool b1, b2, b3;

	b1 = sign(pt, v1, v2) < 0.0;
	b2 = sign(pt, v2, v3) < 0.0;
	b3 = sign(pt, v3, v1) < 0.0;

	return ((b1 == b2) && (b2 == b3));
}


void main()	{
	vec4		inputPixelColor = IMG_THIS_PIXEL(inputImage);
	vec2		loc = isf_FragNormCoord;
	bool		drawPixel =  false;

	if ((PointInTriangle(loc,pt1,pt2,pt3))||(PointInTriangle(loc,pt1,pt4,pt3)))	{
		drawPixel = true;
	}
	
	if (invertMask)
		drawPixel = !drawPixel;
	
	if (maskApplyMode == 0)	{
		inputPixelColor = (drawPixel) ? inputPixelColor : vec4(0.0);
	}
	//	in this mode only the alpha is changed; the rgb remains intact and may still be visible if another filter adjusts the alpha again
	else if (maskApplyMode == 1)	{
		inputPixelColor.a = (drawPixel) ? inputPixelColor.a : 0.0;
	}
	else if (maskApplyMode == 2)	{
		inputPixelColor = (drawPixel) ? vec4(1.0) : vec4(0.0);
	}

	gl_FragColor = inputPixelColor;
}
