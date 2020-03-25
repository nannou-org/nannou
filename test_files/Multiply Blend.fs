/*
{
  "ISFVSN" : "2",
  "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/multiply_blend.glsl",
  "DESCRIPTION": "",
  "CATEGORIES" : [
    "Dissolve"
  ],
  "INPUTS" : [
    {
      "NAME" : "startImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "endImage",
      "TYPE" : "image"
    },
    {
      "MAX" : 1,
      "DEFAULT" : 0,
      "MIN" : 0,
      "TYPE" : "float",
      "NAME" : "progress"
    }
  ]
}
*/



vec4 getFromColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(startImage, inUV);
}
vec4 getToColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(endImage, inUV);
}



// Author: Fernando Kuteken
// License: MIT

vec4 blend(vec4 a, vec4 b) {
  return a * b;
}

vec4 transition (vec2 uv) {
  
  vec4 blended = blend(getFromColor(uv), getToColor(uv));
  
  if (progress < 0.5)
    return mix(getFromColor(uv), blended, 2.0 * progress);
  else
    return mix(blended, getToColor(uv), 2.0 * progress - 1.0);
}




void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}