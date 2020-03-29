/*
{
  "CATEGORIES" : [
    "Color Effect"
  ],
  "DESCRIPTION" : "Tints the corners of the image in different colors.",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "color1",
      "TYPE" : "color",
      "DEFAULT" : [
        1,
        0,
        0,
        1
      ]
    },
    {
      "NAME" : "color2",
      "TYPE" : "color",
      "DEFAULT" : [
        0,
        1,
        0,
        1
      ]
    },
    {
      "NAME" : "color3",
      "TYPE" : "color",
      "DEFAULT" : [
        0,
        0,
        1,
        1
      ]
    },
    {
      "NAME" : "color4",
      "TYPE" : "color",
      "DEFAULT" : [
        1,
        1,
        1,
        1
      ]
    },
    {
      "NAME" : "rotationAngle",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0,
      "MIN" : 0
    }
  ],
  "CREDIT" : "VIDVOX"
}
*/

const float pi = 3.1415926535897932384626433832795;

vec2 rotatePointNorm(vec2 pt, float rot)	{
	vec2 returnMe = pt;

	float r = distance(vec2(0.50), returnMe);
	float a = atan((returnMe.y-0.5),(returnMe.x-0.5));

	returnMe.x = r * cos(a + 2.0 * pi * rot - pi) + 0.5;
	returnMe.y = r * sin(a + 2.0 * pi * rot - pi) + 0.5;
	
	returnMe = returnMe;
	
	return returnMe;
}

void main()	{
	vec2		pt = isf_FragNormCoord;
	vec4		srcPixel = IMG_NORM_PIXEL(inputImage,pt);
	vec4		dist = vec4(0.0);
	pt = rotatePointNorm(pt,rotationAngle+0.5);
	dist.r = max(1.0-distance(vec2(0.0,0.0),pt),0.0);
	dist.g = max(1.0-distance(vec2(1.0,0.0),pt),0.0);
	dist.b = max(1.0-distance(vec2(0.0,1.0),pt),0.0);
	dist.a = max(1.0-distance(vec2(1.0,1.0),pt),0.0);
	
	float		luma1 = (srcPixel.r+srcPixel.g+srcPixel.b)/3.0;
	vec4		resultPixel = (color1 * dist.r + color2 * dist.g + color3 * dist.b + color4 * dist.a) / (dist.r + dist.g + dist.b + dist.a);
	float		luma2 = (resultPixel.r+resultPixel.g+resultPixel.b)/3.0;
	resultPixel.rgb *= luma1 / luma2;
	resultPixel.a *= srcPixel.a;
	
	gl_FragColor = resultPixel;
}
