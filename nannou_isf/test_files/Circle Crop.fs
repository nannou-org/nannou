/*{
    "CATEGORIES": [
        "Wipe"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/CircleCrop.glsl",
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
                0,
                0,
                1
            ],
            "NAME": "bgcolor",
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



// License: MIT
// Author: fkuteken
// ported by gre from https://gist.github.com/fkuteken/f63e3009c1143950dee9063c3b83fb88

float ratio = RENDERSIZE.x/RENDERSIZE.y;
vec2 ratio2 = vec2(1.0, 1.0 / ratio);
float s = pow(2.0 * abs(progress - 0.5), 3.0);

vec4 transition(vec2 p) {
  float dist = length((vec2(p) - 0.5) * ratio2);
  return mix(
    progress < 0.5 ? getFromColor(p) : getToColor(p), // branching is ok here as we statically depend on progress uniform (branching won't change over pixels)
    bgcolor,
    step(s, dist)
  );
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}