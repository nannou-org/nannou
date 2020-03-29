/*{
    "CATEGORIES": [
        "Geometry"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "DEFAULT": 0.5,
            "LABEL": "Saturation",
            "MAX": 1,
            "MIN": 0,
            "NAME": "saturation",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Brightness",
            "MAX": 1,
            "MIN": 0,
            "NAME": "brightness",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Mix Amount",
            "MAX": 1,
            "MIN": 0,
            "NAME": "mixAmount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Mask Shape Mode",
            "LABELS": [
                "Random",
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
                3,
                4
            ]
        },
        {
            "DEFAULT": 0,
            "LABEL": "Anchor To Bottom",
            "NAME": "anchorToBottom",
            "TYPE": "bool"
        },
        {
            "LABEL": "Reset",
            "NAME": "resetImage",
            "TYPE": "event"
        }
    ],
    "ISFVSN": "2",
    "KEYWORDS": [
        "Abstract",
        "Geometric"
    ],
    "PASSES": [
        {
            "DESCRIPTION": "Holds the last render state for drawing over",
            "PERSISTENT": true,
            "TARGET": "lastState"
        }
    ]
}
*/

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

vec4 rand4(vec4 co)	{
	vec4	returnMe = vec4(0.0);
	returnMe.r = rand(co.rg);
	returnMe.g = rand(co.gb);
	returnMe.b = rand(co.ba);
	returnMe.a = rand(co.rb);
	return returnMe;
}

vec3 hsv2rgb(vec3 c)	{
	vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
	vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
	return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

const float pi = 3.14159265359;


vec2 rotatePoint(vec2 pt, float angle, vec2 center)
{
	vec2 returnMe;
	float s = sin(angle * pi);
	float c = cos(angle * pi);

	returnMe = pt;

	// translate point back to origin:
	returnMe.x -= center.x;
	returnMe.y -= center.y;

	// rotate point
	float xnew = returnMe.x * c - returnMe.y * s;
	float ynew = returnMe.x * s + returnMe.y * c;

	// translate point back:
	returnMe.x = xnew + center.x;
	returnMe.y = ynew + center.y;
	return returnMe;
}

float sign(vec2 p1, vec2 p2, vec2 p3)
{
	return (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y);
}

bool RotatedPointInTriangle(vec2 pt, vec2 v1, vec2 v2, vec2 v3, vec2 center)
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


void main()	{
	vec2	loc = isf_FragNormCoord.xy;
	vec4	returnMe = (resetImage) ? vec4(0.0) : IMG_NORM_PIXEL(lastState,loc);
	vec4	seeds1 = TIME * vec4(0.2123,0.34517,0.53428,0.7431);
	vec4	randCoords = rand4(seeds1);
	if (anchorToBottom == true)	{
		randCoords.y = 0.0;
	}
	int		shapeMode = (maskShapeMode != 0) ? maskShapeMode - 1 : int(floor(3.99 * rand(vec2(TIME+0.213,0.43*TIME+0.831))));
	float	isInShape = isPointInShape(loc,shapeMode,randCoords);
	
	if (isInShape > 0.0)	{
		float		randHue = rand(vec2(TIME,0.32234));
		vec4		newColor = vec4(0.0);
		newColor.rgb = hsv2rgb(vec3(randHue,saturation,brightness));
		newColor.a = 1.0;
		returnMe = mix(returnMe,newColor,mixAmount);
	}
	
	gl_FragColor = returnMe;
}
