/*{
    "CATEGORIES": [
        "Geometry"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "spacing",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.05,
            "MAX": 1,
            "MIN": 0,
            "NAME": "line_width",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": -1,
            "NAME": "angle",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "shift",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                1,
                1,
                1,
                1
            ],
            "NAME": "color1",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0,
                0,
                0,
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


float pattern() {
	float s = sin(angle * pi);
	float c = cos(angle * pi);
	vec2 tex = isf_FragNormCoord * RENDERSIZE;
	float spaced = RENDERSIZE.y * spacing;
	vec2 point = vec2( c * tex.x - s * tex.y, s * tex.x + c * tex.y ) * max(1.0/spaced,0.001);
	float d = point.y;
	float w = line_width;
	if (w > spacing)	{
		w = 0.99*spacing;	
	}
	return ( mod(d + shift*spacing + w * 0.5,spacing) );
}


void main() {
	//	determine if we are on a line
	//	math goes something like, figure out distance to the closest line, then draw color2 if we're within range
	//	y = m*x + b
	//	m = (y1-y0)/(x1-x0) = tan(angle)
	
	vec4 out_color = color2;
	float w = line_width;
	if (w > spacing)	{
		w = 0.99*spacing;	
	}
	float pat = pattern();
	if ((pat > 0.0)&&(pat <= w))	{
		float percent = (1.0-abs(w-2.0*pat)/w);
		percent = clamp(percent,0.0,1.0);
		out_color = mix(color2,color1,percent);
	}
	
	gl_FragColor = out_color;
}