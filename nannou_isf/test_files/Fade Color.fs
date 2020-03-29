/*{
    "CATEGORIES": [
        "Dissolve"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/fadecolor.glsl",
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
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": 0,
            "NAME": "colorPhase",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0,
                0,
                0,
                1
            ],
            "NAME": "color",
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



// author: gre
// License: MIT
vec4 transition (vec2 uv) {
  return mix(
    mix(color, getFromColor(uv), smoothstep(1.0-colorPhase, 0.0, progress)),
    mix(color, getToColor(uv), smoothstep(    colorPhase, 1.0, progress)),
    progress);
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}