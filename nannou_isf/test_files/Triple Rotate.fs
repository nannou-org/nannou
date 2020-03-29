/*{
    "CATEGORIES": [
        "Geometry Adjustment"
    ],
    "CREDIT": "by VIDVOX",
    "DESCRIPTION": "Performs three different rotations",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "angle1",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "angle2",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "angle3",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "angle4",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.15,
            "MAX": 1,
            "MIN": 0,
            "NAME": "radius1",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.15,
            "MAX": 1,
            "MIN": 0,
            "NAME": "radius2",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.15,
            "MAX": 1,
            "MIN": 0,
            "NAME": "radius3",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

const float pi = 3.14159265359;

void main()	{
	//	'loc' is the location in pixels of this vertex.  we're going to convert this to polar coordinates (radius/angle)
	vec2		loc = RENDERSIZE * vec2(isf_FragNormCoord[0],isf_FragNormCoord[1]);
	//	'r' is the radius- the distance in pixels from 'loc' to the center of the rendering space
	float		r = distance(RENDERSIZE/2.0, loc);
	//	'a' is the angle of the line segment from the center to loc is rotated
	float		a = atan ((loc.y-RENDERSIZE.y/2.0),(loc.x-RENDERSIZE.x/2.0));
	
	//	now modify 'a', and convert the modified polar coords (radius/angle) back to cartesian coords (x/y pixels)
	float	angle = angle1;
	float	minSide = min(RENDERSIZE.x,RENDERSIZE.y);
	if (r > (radius1 + radius2 + radius3)*minSide)
		angle = angle4;
	else if (r > (radius1 + radius2)*minSide)
		angle = angle3;
	else if (r > radius1 * minSide)
		angle = angle2;
	loc.x = r * cos(a + 2.0 * pi * angle);
	loc.y = r * sin(a + 2.0 * pi * angle);
	
	loc = loc / RENDERSIZE + vec2(0.5);
	
	if ((loc.x < 0.0)||(loc.y < 0.0)||(loc.x > 1.0)||(loc.y > 1.0))	{
		gl_FragColor = vec4(0.0);
	}
	else	{
		gl_FragColor = IMG_NORM_PIXEL(inputImage,loc);
	}
}