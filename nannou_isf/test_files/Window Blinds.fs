/*
{
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
      "NAME" : "progress",
      "MIN" : 0,
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0
    }
  ],
  "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/windowblinds.glsl",
  "DESCRIPTION": "",
  "ISFVSN" : "2"
}
*/



vec4 getFromColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(startImage, inUV);
}
vec4 getToColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(endImage, inUV);
}



// Author: Fabien Benetou
// License: MIT

vec4 transition (vec2 uv) {
  float t = progress;
  
  if (mod(floor(uv.y*100.*progress),2.)==0.)
    t*=2.-.5;
  
  return mix(
    getFromColor(uv),
    getToColor(uv),
    mix(t, progress, smoothstep(0.8, 1.0, progress))
  );
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}