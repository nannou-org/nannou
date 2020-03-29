/*
{
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
      "MAX" : 1,
      "TYPE" : "float",
      "MIN" : 0,
      "NAME" : "progress",
      "DEFAULT" : 0
    }
  ],
  "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/fade.glsl",
  "DESCRIPTION": "",
  "CATEGORIES" : [
    "Dissolve"
  ],
  "ISFVSN" : "2"
}
*/



vec4 getFromColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(startImage, inUV);
}
vec4 getToColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(endImage, inUV);
}



// author: gre
// license: MIT

vec4 transition (vec2 uv) {
  return mix(
    getFromColor(uv),
    getToColor(uv),
    progress
  );
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}