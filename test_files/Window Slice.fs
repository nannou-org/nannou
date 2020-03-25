/*{
    "CATEGORIES": [
        "Wipe"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/windowslice.glsl",
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
            "DEFAULT": 10,
            "MAX": 100,
            "MIN": 0,
            "NAME": "count",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "smoothness",
            "TYPE": "float"
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


vec4 transition (vec2 p) {
  float pr = smoothstep(-smoothness, 0.0, p.x - progress * (1.0 + smoothness));
  float s = step(pr, fract(count * p.x));
  return mix(getFromColor(p), getToColor(p), s);
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}