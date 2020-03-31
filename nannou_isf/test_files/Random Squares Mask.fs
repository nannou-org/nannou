/*
{
  "CATEGORIES" : [
    "Masking"
  ],
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "width",
      "TYPE" : "float",
      "DEFAULT" : 0.125
    },
    {
      "NAME" : "offset",
      "TYPE" : "point2D",
      "DEFAULT" : [
        0,
        0
      ],
      "MIN": [
      	0,
      	0
      ],
      "MAX": [
      	1,
      	1
      ]
    },
    {
      "NAME" : "alpha1",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0,
      "MIN" : 0
    },
    {
      "NAME" : "alpha2",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 1,
      "MIN" : 0
    },
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "seed1",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.241,
      "MIN" : 0
    },
    {
      "NAME" : "randomThreshold",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.5,
      "MIN" : 0
    }
  ],
  "CREDIT" : "by VIDVOX"
}
*/


//	glsl doesn't include random functions
//	this is a pseudo-random function
float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}


void main() {
	//	determine if we are on an even or odd line
	//	math goes like..
	//	mod(((coord+offset) / width),2)
	
	
	vec4 out_color = IMG_THIS_NORM_PIXEL(inputImage);
	float alphaAdjust = alpha2;
	vec2 coord = isf_FragNormCoord * RENDERSIZE;
	vec2 shift = offset * RENDERSIZE;
	float size = width * RENDERSIZE.x;
	vec2 gridIndex = vec2(0.0);

	if (size == 0.0)	{
		alphaAdjust = alpha1;
	}
	else {
		gridIndex = floor((shift + coord) / size);
		float value = rand(seed1*gridIndex);
		if (value < randomThreshold)
			alphaAdjust = alpha1;
	}
	
	out_color.a *= alphaAdjust;
	
	gl_FragColor = out_color;
}