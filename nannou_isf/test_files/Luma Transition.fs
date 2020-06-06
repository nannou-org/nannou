/*
{
  "ISFVSN" : "2",
  "CATEGORIES" : [
    "Dissolve"
  ],
  "INPUTS" : [
    {
      "TYPE" : "image",
      "NAME" : "startImage"
    },
    {
      "NAME" : "endImage",
      "TYPE" : "image"
    },
    {
      "TYPE" : "float",
      "MIN" : 0,
      "DEFAULT" : 0,
      "NAME" : "progress",
      "MAX" : 1
    },
    {
      "NAME" : "luma",
      "TYPE" : "image"
    }
  ],
  "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/luma.glsl",
  "DESCRIPTION" : "Automatically converted from https://gl-transitions.com/"
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


vec4 transition(vec2 uv) {
  return mix(
    getToColor(uv),
    getFromColor(uv),
    step(progress, IMG_NORM_PIXEL(luma, uv).r-0.001)
  );
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}