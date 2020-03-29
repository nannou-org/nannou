/*{
    "CATEGORIES": [
        "Distortion"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/CrazyParametricFun.glsl",
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
            "DEFAULT": 1,
            "MAX": 10,
            "MIN": 0,
            "NAME": "b",
            "TYPE": "float"
        },
        {
            "NAME": "smoothness",
            "TYPE": "float"
        },
        {
            "DEFAULT": 120,
            "MAX": 360,
            "MIN": 0,
            "NAME": "amplitude",
            "TYPE": "float"
        },
        {
            "DEFAULT": 4,
            "MAX": 10,
            "MIN": 0,
            "NAME": "a",
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



// Author: mandubian
// License: MIT


vec4 transition(vec2 uv) {
  vec2 p = uv.xy / vec2(1.0).xy;
  vec2 dir = p - vec2(.5);
  float dist = length(dir);
  float x = (a - b) * cos(progress) + b * cos(progress * ((a / b) - 1.) );
  float y = (a - b) * sin(progress) - b * sin(progress * ((a / b) - 1.));
  vec2 offset = dir * vec2(sin(progress  * dist * amplitude * x), sin(progress * dist * amplitude * y)) / smoothness;
  return mix(getFromColor(p + offset), getToColor(p), smoothstep(0.2, 1.0, progress));
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}