/*{
    "CATEGORIES": [
        "Wipe"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/squeeze.glsl",
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
            "DEFAULT": 0.04,
            "MAX": 1,
            "MIN": 0,
            "NAME": "colorSeparation",
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
 
 
vec4 transition (vec2 uv) {
  float y = 0.5 + (uv.y-0.5) / (1.0-progress);
  if (y < 0.0 || y > 1.0) {
     return getToColor(uv);
  }
  else {
    vec2 fp = vec2(uv.x, y);
    vec2 off = progress * vec2(0.0, colorSeparation);
    vec4 c = getFromColor(fp);
    vec4 cn = getFromColor(fp - off);
    vec4 cp = getFromColor(fp + off);
    return vec4(cn.r, c.g, cp.b, c.a);
  }
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}