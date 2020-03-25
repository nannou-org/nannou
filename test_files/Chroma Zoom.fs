/*
{
  "CATEGORIES" : [
    "Color Effect",
    "Stylize"
  ],
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "master_zoom",
      "TYPE" : "float",
      "MAX" : 2,
      "DEFAULT" : 1,
      "MIN" : 0.1
    },
    {
      "NAME" : "red_zoom",
      "TYPE" : "float",
      "MAX" : 1.5,
      "DEFAULT" : 1,
      "MIN" : 1
    },
    {
      "NAME" : "green_zoom",
      "TYPE" : "float",
      "MAX" : 1.5,
      "DEFAULT" : 1,
      "MIN" : 1
    },
    {
      "NAME" : "blue_zoom",
      "TYPE" : "float",
      "MAX" : 1.5,
      "DEFAULT" : 1,
      "MIN" : 1
    },
    {
      "NAME" : "center",
      "TYPE" : "point2D",
      "MAX" : [
        1,
        1
      ],
      "DEFAULT" : [
        0.5,
        0.5
      ],
      "MIN" : [
        0,
        0
      ]
    }
  ],
  "CREDIT" : "by toneburst"
}
*/

void main() {
	vec2		loc;
	vec2		modifiedCenter;
	
	loc = isf_FragNormCoord;
	modifiedCenter = center;
	
	vec2 locR = (loc - modifiedCenter)*(1.0/(red_zoom*master_zoom)) + modifiedCenter;
	vec2 locG = (loc - modifiedCenter)*(1.0/(green_zoom*master_zoom)) + modifiedCenter;
	vec2 locB = (loc - modifiedCenter)*(1.0/(blue_zoom*master_zoom)) + modifiedCenter;
	
	vec4 outPix;
	outPix.r = IMG_NORM_PIXEL(inputImage,locR).r;
	outPix.g = IMG_NORM_PIXEL(inputImage,locG).g;
	outPix.b = IMG_NORM_PIXEL(inputImage,locB).b;

	loc.x = (loc.x - modifiedCenter.x)*(1.0/master_zoom) + modifiedCenter.x;
	loc.y = (loc.y - modifiedCenter.y)*(1.0/master_zoom) + modifiedCenter.y;
	
	outPix.a = IMG_NORM_PIXEL(inputImage,loc).a;
	gl_FragColor = outPix;
}
