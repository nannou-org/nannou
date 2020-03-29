/*
{
  "ISFVSN" : "2",
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
      "NAME" : "progress",
      "MIN" : 0,
      "MAX" : 1,
      "DEFAULT" : 0
    }
  ],
  "CATEGORIES" : [
    "Distortion"
  ],
  "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/Dreamy.glsl",
  "DESCRIPTION" : "Automatically converted from https://gl-transitions.com/"
}
*/



vec4 getFromColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(startImage, inUV);
}
vec4 getToColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(endImage, inUV);
}



// Author: mikolalysenko
// License: MIT

vec2 offset(float progress, float x, float theta) {
  float phase = progress*progress + progress + theta;
  float shifty = 0.03*progress*cos(10.0*(progress+x));
  return vec2(0, shifty);
}
vec4 transition(vec2 p) {
  return mix(getFromColor(p + offset(progress, p.x, 0.0)), getToColor(p + offset(1.0-progress, p.x, 3.14)), progress);
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}