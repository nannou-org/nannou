/*{
    "CATEGORIES": [
        "Geometry"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "rotation",
            "TYPE": "float"
        },
        {
            "DEFAULT": 2,
            "MAX": 50,
            "MIN": 0.1,
            "NAME": "count",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.125,
            "MAX": 0.25,
            "MIN": 0,
            "NAME": "width",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": 0,
            "NAME": "softness",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0,
                0,
                0,
                0
            ],
            "NAME": "color1",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                1,
                1,
                1,
                1
            ],
            "NAME": "color2",
            "TYPE": "color"
        }
    ],
    "ISFVSN": "2"
}
*/


const float pi = 3.14159265359;


void main() {
	//	determine if we are on an even or odd line
	//	math goes like..
	//	r = a + b*theta
	//	Changing the parameter 'a' will turn the spiral, while 'b' controls the distance between successive turnings.
	
	
	vec4 out_color = color1;
	
	//	convert to polar coordinates
	vec2 loc = vec2(isf_FragNormCoord[0],isf_FragNormCoord[1]);
	loc.y = (loc.y - 0.5) * RENDERSIZE.y / RENDERSIZE.x + 0.5;
	float r = 2.0 * count * distance(vec2(0.5,0.5), loc) + width;
	float theta = atan ((loc.y-0.5),(loc.x-0.5));

	loc.y = r * sin(theta + 2.0 * pi * rotation) + 0.5;

	if (loc.y < 0.5)	{
		theta = theta + 2.0 * pi;
		theta = mod(theta + rotation * 2.0 * pi, 2.0 * pi);
		theta = (theta + 2.0 * pi * floor(r - width));
	}
	else	{
		theta = mod(theta + rotation * 2.0 * pi, 2.0 * pi);
		theta = (theta + 2.0 * pi * floor(r + width));
	}
	
	if (width == 0.0)	{
		out_color = color1;
	}
	else 	{
		float dist = abs(r - theta/(2.0*pi));
		if (dist < width)	{
			if (dist > width * (1.0-softness))	{
				out_color = mix(color2, color1, (dist - width * (1.0-softness))/(width - width * (1.0-softness)));
			}
			else	{
				out_color = color2;
			}
		}
	}
	
	gl_FragColor = out_color;
}