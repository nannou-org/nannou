/*{
    "CATEGORIES": [
        "Distortion"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/pixelize.glsl",
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
            "DEFAULT": 50,
            "MAX": 100,
            "MIN": 0,
            "NAME": "steps",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                20,
                20
            ],
            "MAX": [
                100,
                100
            ],
            "MIN": [
                1,
                1
            ],
            "NAME": "squaresMin",
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



// Author: gre
// License: MIT
// forked from https://gist.github.com/benraziel/c528607361d90a072e98


float d = min(progress, 1.0 - progress);
float dist = steps>0 ? ceil(d * float(steps)) / float(steps) : d;
vec2 squareSize = 2.0 * dist / vec2(squaresMin);

vec4 transition(vec2 uv) {
  vec2 p = dist>0.0 ? (floor(uv / squareSize) + 0.5) * squareSize : uv;
  return mix(getFromColor(p), getToColor(p), progress);
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}