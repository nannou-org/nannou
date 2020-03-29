/*{
    "CATEGORIES": [
        "Wipe"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/PolkaDotsCurtain.glsl",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "NAME": "startImage",
            "TYPE": "image"
        },
        {
            "NAME": "endImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "progress",
            "TYPE": "float"
        },
        {
            "DEFAULT": 20,
            "MAX": 100,
            "MIN": 0,
            "NAME": "dots",
            "TYPE": "float"
        },
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
            "NAME": "center",
            "TYPE": "point2D"
        }
    ],
    "ISFVSN": "2"
}
*/



vec4 getFromColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(startImage, inUV);
}
vec4 getToColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(endImage, inUV);
}



// author: bobylito
// license: MIT
const float SQRT_2 = 1.414213562373;

vec4 transition(vec2 uv) {
  bool nextImage = distance(fract(uv * dots), vec2(0.5, 0.5)) < ( progress / distance(uv, center));
  return nextImage ? getToColor(uv) : getFromColor(uv);
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}