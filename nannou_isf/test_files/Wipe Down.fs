/*
{
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "TYPE" : "image",
      "NAME" : "startImage"
    },
    {
      "TYPE" : "image",
      "NAME" : "endImage"
    },
    {
      "TYPE" : "float",
      "NAME" : "progress",
      "MIN" : 0,
      "MAX" : 1,
      "DEFAULT" : 0
    }
  ],
  "CATEGORIES" : [
    "Wipe"
  ],
  "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/wipeDown.glsl",
  "DESCRIPTION" : "Automatically converted from https://gl-transitions.com/"
}
*/



vec4 getFromColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(startImage, inUV);
}
vec4 getToColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(endImage, inUV);
}



// Author: Jake Nelson
// License: MIT

vec4 transition(vec2 uv) {
  vec2 p=uv.xy/vec2(1.0).xy;
  vec4 a=getFromColor(p);
  vec4 b=getToColor(p);
  return mix(a, b, step(1.0-p.y,progress));
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}