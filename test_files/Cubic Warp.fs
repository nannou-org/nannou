/*{
  "CREDIT": "by carter rosenberg",
  "CATEGORIES": [
    "Distortion Effect"
  ],
  "INPUTS": [
    {
      "NAME": "inputImage",
      "TYPE": "image"
    },
    {
      "NAME": "level",
      "TYPE": "float",
      "MIN": 0,
      "MAX": 100,
      "DEFAULT": 1
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
  ]
}*/

const float pi = 3.14159265359;


#ifndef GL_ES
float distance (vec2 center, vec2 pt)
{
	float tmp = pow(center.x-pt.x,2.0)+pow(center.y-pt.y,2.0);
	return pow(tmp,0.5);
}
#endif


void main() {
	vec2		loc;
	vec2		modifiedCenter;

	loc = isf_FragNormCoord;
	modifiedCenter = center;
   
	// lens distortion coefficient
	float k = -0.15;

	float r2 = (loc.x-modifiedCenter.x) * (loc.x-modifiedCenter.x) + (loc.y-modifiedCenter.y) * (loc.y-modifiedCenter.y);       
	float f = 0.0;

	//only compute the cubic distortion if necessary
	if(level == 0.0){
		f = 1.0 + r2 * k;
	}
	else	{
		f = 1.0 + r2 * (k + level * sqrt(r2));
	};

	float zoom = max(sqrt(level),1.0);

	// get the right pixel for the current position
	loc.x = f*(loc.x-modifiedCenter.x)/zoom+modifiedCenter.x;
	loc.y = f*(loc.y-modifiedCenter.y)/zoom+modifiedCenter.y;

	if ((loc.x < 0.0)||(loc.y < 0.0)||(loc.x > 1.0)||(loc.y > 1.0))	{
		gl_FragColor = vec4(0.0);
	}
	else	{
		gl_FragColor = IMG_NORM_PIXEL(inputImage,loc);
	}
}
