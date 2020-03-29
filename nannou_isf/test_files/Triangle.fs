/*{
    "CATEGORIES": [
        "Geometry"
    ],
    "CREDIT": "by Carter Rosenberg",
    "INPUTS": [
        {
            "DEFAULT": [
                0,
                0
            ],
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
                0.5,
                1
            ],
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
                0
            ],
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
                1,
                1,
                1,
                1
            ],
            "NAME": "fillColor",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0,
                0,
                0,
                0
            ],
            "NAME": "bgColor",
            "TYPE": "color"
        }
    ],
    "ISFVSN": "2"
}
*/


//	functions via http://stackoverflow.com/questions/2049582/how-to-determine-a-point-in-a-triangle

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


void main() {
	vec2 loc = isf_FragNormCoord;
	vec4 outColor = vec4(0.0);
	vec2 point1 = pt1;
	vec2 point2 = pt2;
	vec2 point3 = pt3;
	
	//	determine if we are inside or outside of the triangle
	
	gl_FragColor = mix(bgColor, fillColor, float(PointInTriangle(loc, point1,point2,point3)));;
}