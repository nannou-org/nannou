/*
{
  "CATEGORIES" : [
    "Distortion Effect"
  ],
  "DESCRIPTION" : "Wraps the video into a circular shape",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "inputAngle",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.5,
      "MIN" : 0
    },
    {
      "NAME" : "inputRadius",
      "TYPE" : "float",
      "MAX" : 2,
      "DEFAULT" : 1,
      "MIN" : 0
    },
    {
      "NAME" : "inputCenter",
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
    },
    {
      "NAME" : "mirror",
      "TYPE" : "bool",
      "DEFAULT" : false
    },
    {
      "NAME" : "correctAspect",
      "TYPE" : "bool",
      "DEFAULT" : true
    }
  ],
  "CREDIT" : "VIDVOX"
}
*/


const float tau = 6.28318530718;
const float pi = 3.14159265359;
const float halfpi = 1.57079632679;


void main()	{
	vec4		inputPixelColor;
	if (inputRadius > 0.0)	{
	
		vec2		loc = isf_FragNormCoord.xy;
		vec2		center = inputCenter;
	
		//	account for aspect ratio so we stay circle
		if (correctAspect)	{
			loc.x = loc.x * RENDERSIZE.x/RENDERSIZE.y;
			center.x = center.x * RENDERSIZE.x/RENDERSIZE.y;
		}
		//	translate from polar coords back to cart for this point
		//		the effect translates (x,y) to (r,theta)
		float		r = distance(loc,center);
		float		theta = (inputAngle * tau + pi + atan(loc.x-center.x,loc.y-center.y))/tau;
		
		theta = mod(theta, 1.0);
	
		//loc = vec2((r * 2.0)/inputRadius, theta / pi);
		loc = vec2(theta, (r * 2.0)/inputRadius);
		
		if ((loc.x < 0.0)||(loc.x > 1.0)||(loc.y < 0.0)||(loc.y > 1.0))
			inputPixelColor = vec4(0.0);
		else	{
			loc.y = 1.0 - loc.y;
			if (mirror)	{
				loc.x = (loc.x < 0.5) ? loc.x * 2.0 : 2.0 - loc.x * 2.0;
			}
			inputPixelColor = IMG_NORM_PIXEL(inputImage, loc);
		}
	}
	gl_FragColor = inputPixelColor;
}
