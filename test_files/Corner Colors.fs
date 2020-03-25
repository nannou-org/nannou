/*{
    "CATEGORIES": [
        "Color"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "Generates a gradient that fades between four different colors.",
    "INPUTS": [
        {
            "DEFAULT": [
                1,
                0,
                0,
                1
            ],
            "NAME": "color1",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0,
                1,
                0,
                1
            ],
            "NAME": "color2",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0,
                0,
                1,
                1
            ],
            "NAME": "color3",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                1,
                1,
                1,
                1
            ],
            "NAME": "color4",
            "TYPE": "color"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "rotationAngle",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

const float pi = 3.1415926535897932384626433832795;

vec2 rotatePointNorm(vec2 pt, float rot)	{
	vec2 returnMe = pt;

	float r = distance(vec2(0.50), returnMe);
	float a = atan((returnMe.y-0.5),(returnMe.x-0.5));

	returnMe.x = r * cos(a + 2.0 * pi * rot - pi) + 0.5;
	returnMe.y = r * sin(a + 2.0 * pi * rot - pi) + 0.5;
	
	returnMe = returnMe;
	
	return returnMe;
}

void main()	{
	vec4		inputPixelColor = vec4(0.0);
	vec4		dist = vec4(0.0);
	vec2		pt = isf_FragNormCoord;
	pt = rotatePointNorm(pt,rotationAngle+0.5);
	dist.r = max(1.0-distance(vec2(0.0,0.0),pt),0.0);
	dist.g = max(1.0-distance(vec2(1.0,0.0),pt),0.0);
	dist.b = max(1.0-distance(vec2(0.0,1.0),pt),0.0);
	dist.a = max(1.0-distance(vec2(1.0,1.0),pt),0.0);
	
	inputPixelColor = (color1 * dist.r + color2 * dist.g + color3 * dist.b + color4 * dist.a) / (dist.r + dist.g + dist.b + dist.a);
	
	gl_FragColor = inputPixelColor;
}
