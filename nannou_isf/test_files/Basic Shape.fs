/*{
    "CATEGORIES": [
        "Geometry"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "DEFAULT": [
                1,
                1,
                1,
                1
            ],
            "NAME": "color",
            "TYPE": "color"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Mask Shape Mode",
            "LABELS": [
                "Rectangle",
                "Triangle",
                "Circle",
                "Diamond"
            ],
            "NAME": "maskShapeMode",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2,
                3
            ]
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "Shape Width",
            "MAX": 2,
            "MIN": 0,
            "NAME": "shapeWidth",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "Shape Height",
            "MAX": 2,
            "MIN": 0,
            "NAME": "shapeHeight",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0.5,
                0.5
            ],
            "MAX": [
                1,
                1
            ],
            "MIN": [
                0,
                0
            ],
            "NAME": "center",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": false,
            "LABEL": "Invert Mask",
            "NAME": "invertMask",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Horizontal Repeat",
            "LABELS": [
                "1",
                "2",
                "3",
                "4",
                "5",
                "6",
                "7",
                "8",
                "9"
            ],
            "NAME": "horizontalRepeat",
            "TYPE": "long",
            "VALUES": [
                1,
                2,
                3,
                4,
                5,
                6,
                7,
                8,
                9
            ]
        },
        {
            "DEFAULT": 1,
            "LABEL": "Vertical Repeat",
            "LABELS": [
                "1",
                "2",
                "3",
                "4",
                "5",
                "6",
                "7",
                "8",
                "9"
            ],
            "NAME": "verticalRepeat",
            "TYPE": "long",
            "VALUES": [
                1,
                2,
                3,
                4,
                5,
                6,
                7,
                8,
                9
            ]
        }
    ],
    "ISFVSN": "2"
}
*/



const float pi = 3.14159265359;


vec2 rotatePoint(vec2 pt, float angle, vec2 inCenter)
{
	vec2 returnMe;
	float s = sin(angle * pi);
	float c = cos(angle * pi);

	returnMe = pt;

	// translate point back to origin:
	returnMe.x -= inCenter.x;
	returnMe.y -= inCenter.y;

	// rotate point
	float xnew = returnMe.x * c - returnMe.y * s;
	float ynew = returnMe.x * s + returnMe.y * c;

	// translate point back:
	returnMe.x = xnew + inCenter.x;
	returnMe.y = ynew + inCenter.y;
	return returnMe;
}

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

bool RotatedPointInTriangle(vec2 pt, vec2 v1, vec2 v2, vec2 v3, vec2 inCenter)
{
	bool b1, b2, b3;
	
	vec2 v1r = v1;
	vec2 v2r = v2;
	vec2 v3r = v3;

	b1 = sign(pt, v1r, v2r) < 0.0;
	b2 = sign(pt, v2r, v3r) < 0.0;
	b3 = sign(pt, v3r, v1r) < 0.0;

	return ((b1 == b2) && (b2 == b3));
}


float isPointInShape(vec2 pt, int shape, vec4 shapeCoordinates)	{
	float returnMe = 0.0;
	
	//	rectangle
	if (shape == 0)	{
		if (RotatedPointInTriangle(pt, shapeCoordinates.xy, shapeCoordinates.xy + vec2(0.0, shapeCoordinates.w), shapeCoordinates.xy + vec2(shapeCoordinates.z, 0.0), shapeCoordinates.xy + shapeCoordinates.zw / 2.0))	{
			returnMe = 1.0;
			// soft edge if needed
			if ((pt.x > shapeCoordinates.x) && (pt.x < shapeCoordinates.x)) {
				returnMe = clamp(((pt.x - shapeCoordinates.x) / RENDERSIZE.x), 0.0, 1.0);
				returnMe = pow(returnMe, 0.5);
			}
			else if ((pt.x > shapeCoordinates.x + shapeCoordinates.z) && (pt.x < shapeCoordinates.x + shapeCoordinates.z)) {
				returnMe = clamp(((shapeCoordinates.x + shapeCoordinates.z - pt.x) / RENDERSIZE.x), 0.0, 1.0);
				returnMe = pow(returnMe, 0.5);
			}
		}
		else if (RotatedPointInTriangle(pt, shapeCoordinates.xy + shapeCoordinates.zw, shapeCoordinates.xy + vec2(0.0, shapeCoordinates.w), shapeCoordinates.xy + vec2(shapeCoordinates.z, 0.0), shapeCoordinates.xy + shapeCoordinates.zw / 2.0))	{
			returnMe = 1.0;
			// soft edge if needed
			if ((pt.x > shapeCoordinates.x) && (pt.x < shapeCoordinates.x)) {
				returnMe = clamp(((pt.x - shapeCoordinates.x) / RENDERSIZE.x), 0.0, 1.0);
				returnMe = pow(returnMe, 0.5);
			}
			else if ((pt.x > shapeCoordinates.x + shapeCoordinates.z) && (pt.x < shapeCoordinates.x + shapeCoordinates.z)) {
				returnMe = clamp(((shapeCoordinates.x + shapeCoordinates.z - pt.x) / RENDERSIZE.x), 0.0, 1.0);
				returnMe = pow(returnMe, 0.5);
			}
		}
	}
	//	triangle
	else if (shape == 1)	{
		if (RotatedPointInTriangle(pt, shapeCoordinates.xy, shapeCoordinates.xy + vec2(shapeCoordinates.z / 2.0, shapeCoordinates.w), shapeCoordinates.xy + vec2(shapeCoordinates.z, 0.0), shapeCoordinates.xy + shapeCoordinates.zw / 2.0))	{
			returnMe = 1.0;
		}
	}
	//	oval
	else if (shape == 2)	{
		returnMe = distance(pt, vec2(shapeCoordinates.xy + shapeCoordinates.zw / 2.0));
		if (returnMe < min(shapeCoordinates.z,shapeCoordinates.w) / 2.0)	{
			returnMe = 1.0;
		}
		else	{
			returnMe = 0.0;
		}
	}
	//	diamond
	else if (shape == 3)	{
		if (RotatedPointInTriangle(pt, shapeCoordinates.xy + vec2(0.0, shapeCoordinates.w / 2.0), shapeCoordinates.xy + vec2(shapeCoordinates.z / 2.0, shapeCoordinates.w), shapeCoordinates.xy + vec2(shapeCoordinates.z, shapeCoordinates.w / 2.0), shapeCoordinates.xy + shapeCoordinates.zw / 2.0))	{
			returnMe = 1.0;
		}
		else if (RotatedPointInTriangle(pt, shapeCoordinates.xy + vec2(0.0, shapeCoordinates.w / 2.0), shapeCoordinates.xy + vec2(shapeCoordinates.z / 2.0, 0.0), shapeCoordinates.xy + vec2(shapeCoordinates.z, shapeCoordinates.w / 2.0), shapeCoordinates.xy + shapeCoordinates.zw / 2.0))	{
			returnMe = 1.0;
		}
	}

	return returnMe;	
}



void main() {
	vec4		srcPixel = color;
	vec2		centerPt = center * RENDERSIZE;
	vec2		tmpVec = RENDERSIZE * vec2(shapeWidth,shapeHeight) / 2.0;
	vec4		patternRect = vec4(vec2(centerPt - tmpVec),tmpVec * 2.0);
	vec2		thisPoint = RENDERSIZE * isf_FragNormCoord;
	
	if ((thisPoint.x >= patternRect.x) && (thisPoint.x <= patternRect.x + abs(patternRect.z)))	{
		patternRect.z = patternRect.z / float(horizontalRepeat);
		patternRect.x = patternRect.x + abs(patternRect.z) * floor((thisPoint.x - patternRect.x) / abs(patternRect.z));
	}
	else	{
		patternRect.z = patternRect.z / float(horizontalRepeat);	
	}
	
	if ((thisPoint.y >= patternRect.y) && (thisPoint.y <= patternRect.y + abs(patternRect.w)))	{
		patternRect.w = patternRect.w / float(verticalRepeat);
		patternRect.y = patternRect.y + abs(patternRect.w) * floor((thisPoint.y - patternRect.y) / abs(patternRect.w));
	}
	else	{
		patternRect.w = patternRect.w / float(verticalRepeat);	
	}
	
	float		luminance = isPointInShape(thisPoint.xy, maskShapeMode, patternRect);
	
	if (invertMask)
		luminance = 1.0 - luminance;
	
	srcPixel = color * luminance;
	
	gl_FragColor = srcPixel;
}

