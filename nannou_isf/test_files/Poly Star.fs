/*{
    "CATEGORIES": [
        "Geometry"
    ],
    "CREDIT": "",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "DEFAULT": 5,
            "LABEL": "Point Count",
            "MAX": 20,
            "MIN": 3,
            "NAME": "pointCount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 5,
            "LABEL": "Buldge Amount",
            "MAX": 10,
            "MIN": 0,
            "NAME": "buldge",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.1,
            "LABEL": "Radius Inside",
            "MAX": 1,
            "MIN": 0,
            "NAME": "pointRadiusInside",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.1,
            "LABEL": "Radius Outside",
            "MAX": 1,
            "MIN": 0,
            "NAME": "pointRadiusOutside",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Rotation",
            "MAX": 1,
            "MIN": 0,
            "NAME": "pointRotation",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                1,
                1,
                1,
                1
            ],
            "LABEL": "Star Color",
            "NAME": "starColor",
            "TYPE": "color"
        }
    ],
    "ISFVSN": "2"
}
*/


float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

const float pi = 3.14159265359;

void main()	{
	vec4 inputPixelColor = vec4(0.0);
	float angleInc = 1.0 / pointCount;
	
	vec2 loc = RENDERSIZE * isf_FragNormCoord;
	//	'r' is the radius- the distance in pixels from 'loc' to the center of the rendering space
	float r = distance(RENDERSIZE/2.0, loc);
	r /= max(RENDERSIZE.x,RENDERSIZE.y);
	//	'a' is the angle of the line segment from the center to loc is rotated
	float a = atan ((loc.y-RENDERSIZE.y/2.0),(loc.x-RENDERSIZE.x/2.0));
	a += pi;
	a /= 2.0*pi;
	a += pointRotation;
	//	'at' is the angle time
	float at = mod(a,angleInc) / angleInc;
	//	'rd' is the radius at this angle for the shape 
	float starness = 10.0 - buldge;
	float rd = pointRadiusOutside * pow(at,starness) + pointRadiusInside;
	inputPixelColor = (r < rd) ? starColor : vec4(0.0);
	at = 1.0 - at;
	rd = pointRadiusOutside * pow(at,starness) + pointRadiusInside;
	inputPixelColor = (r < rd) ? starColor : inputPixelColor;
	
	gl_FragColor = inputPixelColor;
}
