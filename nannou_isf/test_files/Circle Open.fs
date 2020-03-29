/*{
    "CATEGORIES": [
        "Wipe"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/circleopen.glsl",
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
            "DEFAULT": 0.3,
            "MAX": 1,
            "MIN": 0,
            "NAME": "smoothness",
            "TYPE": "float"
        },
        {
            "DEFAULT": true,
            "NAME": "opening",
            "TYPE": "bool"
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



// author: gre
// License: MIT

const vec2 center = vec2(0.5, 0.5);
const float SQRT_2 = 1.414213562373;

vec4 transition (vec2 uv) {
  float x = opening ? progress : 1.-progress;
  float m = smoothstep(-smoothness, 0.0, SQRT_2*distance(center, uv) - x*(1.+smoothness));
  return mix(getFromColor(uv), getToColor(uv), opening ? 1.-m : m);
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}