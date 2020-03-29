/*
{
  "CATEGORIES" : [
    "Distortion Effect", "Noise"
  ],
  "DESCRIPTION" : "Displaces image randomly",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "displaceX",
      "TYPE" : "float",
      "MAX" : 0.1,
      "DEFAULT" : 0.0,
      "MIN" : 0.0,
      "LABEL" : "Displace X"
    },
    {
      "NAME" : "displaceY",
      "TYPE" : "float",
      "MAX" : 0.1,
      "DEFAULT" : 0.0,
      "MIN" : 0.0,
      "LABEL" : "Displace Y"
    },
    {
      "NAME" : "detailX",
      "TYPE" : "float",
      "MAX" : 1.0,
      "DEFAULT" : 0.1,
      "MIN" : 0.0,
      "LABEL" : "Detail X"
    },
    {
      "NAME" : "detailY",
      "TYPE" : "float",
      "MAX" : 1.0,
      "DEFAULT" : 0.1,
      "MIN" : 0.0,
      "LABEL" : "Detail Y"
    },
    {
      "NAME" : "updateTime",
      "TYPE" : "float",
      "MAX" : 0.1,
      "DEFAULT" : 0.01,
      "MIN" : 0.0,
      "LABEL" : "Frequency"
    }
  ],
  "CREDIT" : "VIDVOX"
}
*/


float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

void main()	{

	vec4 inputPixelColor;
	vec2 uv = isf_FragNormCoord.xy;
	float wobbleTime = (updateTime == 0.0) ? TIME : floor(TIME/updateTime);
	vec2 waveLoc = fract((uv)*vec2(detailX, detailY));
	float val1 = rand(vec2(waveLoc.x, wobbleTime));
	float val2 = rand(vec2(waveLoc.y, wobbleTime+1.0));
	vec2 wave = vec2(val1, val2)-0.5;
	wave *= vec2(displaceY, displaceX);

	inputPixelColor = IMG_NORM_PIXEL(inputImage, uv + wave.yx);

	gl_FragColor = inputPixelColor;
}
