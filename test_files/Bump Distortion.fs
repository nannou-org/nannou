/*{
    "CATEGORIES": [
        "Distortion Effect"
    ],
    "CREDIT": "by carter rosenberg",
    "DESCRIPTION": "Bends and distorts the image",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": -1,
            "NAME": "level",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": 0,
            "NAME": "radius",
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

#ifndef GL_ES
float distance (vec2 inCenter, vec2 pt)
{
	float tmp = pow(inCenter.x-pt.x,2.0)+pow(inCenter.y-pt.y,2.0);
	return pow(tmp,0.5);
}
#endif

void main() {
	vec2		uv = isf_FragNormCoord.xy;
	vec2		texSize = RENDERSIZE.xy;
	vec2		tc = uv * texSize;
	vec2		modifiedCenter = center * texSize;
	float		r = distance(modifiedCenter, tc);
	float		a = atan ((tc.y-modifiedCenter.y),(tc.x-modifiedCenter.x));
	float		radius_sized = radius * length(RENDERSIZE);
	
	tc -= modifiedCenter;

	if (r < radius_sized) 	{
		float percent = 1.0-(radius_sized - r) / radius_sized;
		if (level>=0.0)	{
			percent = percent * percent;
			tc.x = r*pow(percent,level) * cos(a);
			tc.y = r*pow(percent,level) * sin(a);
		}
		else	{
			float adjustedLevel = level/2.0;
			tc.x = r*pow(percent,adjustedLevel) * cos(a);
			tc.y = r*pow(percent,adjustedLevel) * sin(a);		
		}
	}
	tc += modifiedCenter;
	vec2 loc = tc / texSize;

	if ((loc.x < 0.0)||(loc.y < 0.0)||(loc.x > 1.0)||(loc.y > 1.0))	{
		gl_FragColor = vec4(0.0);
	}
	else	{
		gl_FragColor = IMG_NORM_PIXEL(inputImage, loc);
	}
}
