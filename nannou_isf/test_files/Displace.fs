/*
{
  "CATEGORIES" : [
    "Distortion Effect"
  ],
  "DESCRIPTION" : "Simple Displace",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "displaceImage",
      "TYPE" : "image",
      "LABEL" : "displace image"
    },
    {
      "NAME" : "uDisplaceAmt",
      "TYPE" : "float",
      "MAX" : 2,
      "DEFAULT" : 0,
      "MIN" : -2
    },
    {
      "LABELS" : [
        "Luma",
        "R",
        "G",
        "B",
        "A"
      ],
      "NAME" : "xComponent",
      "TYPE" : "long",
      "DEFAULT" : 1,
      "VALUES" : [
        0,
        1,
        2,
        3,
        4
      ]
    },
    {
      "LABELS" : [
        "Luma",
        "R",
        "G",
        "B",
        "A"
      ],
      "NAME" : "yComponent",
      "TYPE" : "long",
      "DEFAULT" : 2,
      "VALUES" : [
        0,
        1,
        2,
        3,
        4
      ]
    },
    {
      "NAME" : "relativeShift",
      "TYPE" : "bool",
      "DEFAULT" : 0
    }
  ],
  "CREDIT" : "by @colin_movecraft"
}
*/



void main(){
	vec2	p = isf_FragNormCoord.xy;
	vec4	displacePixel = IMG_NORM_PIXEL(displaceImage, p);
    
    float	r = (displacePixel.r);
    float	g = (displacePixel.g);
    float	b = (displacePixel.b);
    float	a = (displacePixel.a);
    float	avg = (r+g+b)/3.0;
    
    vec2	displace = vec2(avg,avg);
    if (xComponent==1)
    	displace.x = r;
	else if (xComponent==2)
    	displace.x = g;
    else if (xComponent==3)
    	displace.x = b;
	else if (xComponent==4)
    	displace.x = a;

    if (yComponent==1)
    	displace.y = r;
	else if (yComponent==2)
    	displace.y = g;
    else if (yComponent==3)
    	displace.y = b;
	else if (yComponent==4)
    	displace.y = a;
    
    displace = (relativeShift) ? 2.0 * (displace - vec2(0.5)) : displace;
    displace *= uDisplaceAmt;
    
	gl_FragColor = IMG_NORM_PIXEL(inputImage,p+displace);
}