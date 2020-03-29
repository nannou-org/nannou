/*
{
  "CATEGORIES" : [
    "Pattern", "Color"
  ],
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "width",
      "TYPE" : "float",
      "DEFAULT" : 0.25
    },
    {
      "NAME" : "offset",
      "TYPE" : "point2D",
      "DEFAULT" : [
        0,
        0
      ],
      "MIN" : [
      	0,
      	0
      ],
      "MAX" : [
      	1,
      	1
      ]
    },
    {
      "NAME" : "color1",
      "TYPE" : "color",
      "DEFAULT" : [
        1,
        1,
        1,
        1
      ]
    },
    {
      "NAME" : "color2",
      "TYPE" : "color",
      "DEFAULT" : [
        0,
        0,
        0,
        1
      ]
    },
    {
      "NAME" : "splitPos",
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
  "CREDIT" : "by VIDVOX"
}
*/



void main() {
	//	determine if we are on an even or odd line
	//	math goes like..
	//	mod(((coord+offset) / width),2)
	
	
	vec4		out_color = color2;
	float		size = width * RENDERSIZE.x;

	if (size == 0.0)	{
		out_color = color1;
	}
	else if ((mod(((gl_FragCoord.x+(offset.x*RENDERSIZE.x)) / size),2.0) < 2.0 * splitPos.x)&&(mod(((gl_FragCoord.y+(offset.y*RENDERSIZE.y)) / size),2.0) > 2.0 * splitPos.y))	{
		out_color = color1;
	}
	else if ((mod(((gl_FragCoord.x+(offset.x*RENDERSIZE.x)) / size),2.0) > 2.0 * splitPos.x)&&(mod(((gl_FragCoord.y+(offset.y*RENDERSIZE.y)) / size),2.0) < 2.0 * splitPos.y))	{
		out_color = color1;
	}
	
	gl_FragColor = out_color;
}