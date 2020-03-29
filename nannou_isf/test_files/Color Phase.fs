/*{
    "CATEGORIES": [
        "Dissolve"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/colorphase.glsl",
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
            "DEFAULT": [
                0,
                0.19607843137254902,
                0.39215686274509803,
                0
            ],
            "NAME": "fromStep",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0.6,
                0.8,
                1,
                1
            ],
            "NAME": "toStep",
            "TYPE": "color"
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

// Usage: fromStep and toStep must be in [0.0, 1.0] range 
// and all(fromStep) must be < all(toStep)


vec4 transition (vec2 uv) {
  vec4 a = getFromColor(uv);
  vec4 b = getToColor(uv);
  return mix(a, b, smoothstep(fromStep, toStep, vec4(progress)));
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}