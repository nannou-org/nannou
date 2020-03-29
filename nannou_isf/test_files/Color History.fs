/*
{
  "CATEGORIES" : [
    "Color"
  ],
  "DESCRIPTION" : "Displays Data Over Time",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "LABELS" : [
        "Color",
        "Lines"
      ],
      "NAME" : "displayMode",
      "TYPE" : "long",
      "DEFAULT" : 0,
      "VALUES" : [
        0,
        1
      ]
    },
    {
      "NAME" : "data",
      "TYPE" : "color",
      "DEFAULT" : [
        0.95,
        0.25,
        0,
        1
      ]
    }
  ],
  "PASSES" : [
    {
      "PERSISTENT" : true,
      "WIDTH" : "$WIDTH",
      "HEIGHT" : "1",
      "TARGET" : "dataHistory",
      "FLOAT" : true
    },
    {

    }
  ],
  "CREDIT" : "VIDVOX"
}
*/

void main()	{
	vec4		inputPixelColor = vec4(0.0);
	if (PASSINDEX == 0)	{
		vec2	loc = gl_FragCoord.xy;
		if (floor(loc.x) == 0.0)	{
			inputPixelColor = data;
		}
		else	{
			loc.x = loc.x - 1.0;
			inputPixelColor = IMG_PIXEL(dataHistory,loc);
		}
	}
	else	{
		vec2	loc = gl_FragCoord.xy;
		vec4	val = IMG_PIXEL(dataHistory,loc);
		if (displayMode == 0)	{
			inputPixelColor = val;
		}
		else if (displayMode == 1)	{
			float	tmp = floor(val.r * RENDERSIZE.y);
			inputPixelColor.a = val.a;
			if (floor(loc.y) == tmp)	{
				inputPixelColor.r = 1.0;
				inputPixelColor.a = 1.0;
			}
			tmp = floor(val.g * RENDERSIZE.y);
			if (floor(loc.y) == tmp)	{
				inputPixelColor.g = 1.0;
				inputPixelColor.a = 1.0;
			}
			tmp = floor(val.b * RENDERSIZE.y);
			if (floor(loc.y) == tmp)	{
				inputPixelColor.b = 1.0;
				inputPixelColor.a = 1.0;
			}
		}
		
	}
	gl_FragColor = inputPixelColor;
}
