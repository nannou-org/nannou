/*{
    "CATEGORIES": [
        "Geometry Adjustment",
        "Utility"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "angle",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

#if __VERSION__ <= 120
varying vec2 translated_coord;
#else
in vec2 translated_coord;
#endif

void main() {
	vec2 loc = translated_coord;
	//	if out of range draw black
	if ((loc.x < 0.0)||(loc.y < 0.0)||(loc.x > 1.0)||(loc.y > 1.0))	{
		gl_FragColor = vec4(0.0);
	}
	else	{
		gl_FragColor = IMG_NORM_PIXEL(inputImage,loc);
	}
}
