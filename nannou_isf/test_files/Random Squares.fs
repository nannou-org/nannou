/*{
    "CATEGORIES": [
        "Dissolve"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/randomsquares.glsl",
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
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "smoothness",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                10,
                10
            ],
            "MAX": [
                100,
                100
            ],
            "MIN": [
                0,
                0
            ],
            "NAME": "size",
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

 
float rand (vec2 co) {
  return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

vec4 transition(vec2 p) {
  float r = rand(floor(vec2(size) * p));
  float m = smoothstep(0.0, -smoothness, r - (progress * (1.0 + smoothness)));
  return mix(getFromColor(p), getToColor(p), m);
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}