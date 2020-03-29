/*{
    "CATEGORIES": [
        "Distortion Effect"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "radius",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 10,
            "MIN": -10,
            "NAME": "amount",
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
        }
    ],
    "ISFVSN": "2"
}
*/


const float pi = 3.14159265359;


void main (void)
{
	vec2 uv = vec2(isf_FragNormCoord[0],isf_FragNormCoord[1]);
	vec2 texSize = RENDERSIZE;
	vec2 tc = uv * texSize;
	float radius_sized = radius * max(RENDERSIZE.x,RENDERSIZE.y);
	tc -= (center * RENDERSIZE);
	float dist = length(tc);
	if (dist < radius_sized) 	{
		float percent = (radius_sized - dist) / radius_sized;
		float theta = percent * percent * amount * 2.0 * pi;
		float s = sin(theta);
		float c = cos(theta);
		tc = vec2(dot(tc, vec2(c, -s)), dot(tc, vec2(s, c)));
	}
	tc += (center * RENDERSIZE);
	vec2 loc = tc / texSize;
	vec4 color = IMG_NORM_PIXEL(inputImage, loc);

	if ((loc.x < 0.0)||(loc.y < 0.0)||(loc.x > 1.0)||(loc.y > 1.0))	{
		gl_FragColor = vec4(0.0);
	}
	else	{
		gl_FragColor = color;
	}
}