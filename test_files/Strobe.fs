/*
{
  "CATEGORIES" : [
    "Color Effect"
  ],
  "DESCRIPTION" : "",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "strobeRate",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0,
      "LABEL" : "Strobe Rate",
      "MIN" : 0
    },
    {
      "LABELS" : [
        "Invert",
        "Color"
      ],
      "NAME" : "strobeMode",
      "TYPE" : "long",
      "LABEL" : "Strobe Mode",
      "VALUES" : [
        0,
        1
      ]
    },
    {
      "NAME" : "strobeColor",
      "TYPE" : "color",
      "DEFAULT" : [
        1,
        1,
        1,
        1
      ],
      "LABEL" : "Strobe Color"
    }
  ],
  "PASSES" : [
    {
      "WIDTH" : "1",
      "DESCRIPTION" : "this buffer stores the last frame's time offset in the first component of its only pixel- note that it's requesting a FLOAT target buffer...",
      "HEIGHT" : "1",
      "TARGET" : "lastState",
      "PERSISTENT" : true
    },
    {

    }
  ],
  "CREDIT" : "by VIDVOX"
}
*/



void main()
{
	//	if this is the first pass, i'm going to read the position from the "lastPosition" image, and write a new position based on this and the hold variables
	if (PASSINDEX == 0)	{
		vec4		srcPixel = IMG_PIXEL(lastState,vec2(0.5));
		//	i'm only using the X, which is the last render time we reset
		if (strobeRate == 0.0)	{
			srcPixel.r = (srcPixel.r == 0.0) ? 1.0 : 0.0;
		}
		else	{
			srcPixel.r = (mod(TIME, strobeRate) <= strobeRate / 2.0) ? 1.0 : 0.0;
		}
		gl_FragColor = srcPixel;
	}
	//	else this isn't the first pass- read the position value from the buffer which stores it
	else	{
		vec4 lastStateVector = IMG_PIXEL(lastState,vec2(0.5));
		vec4 srcPixel = IMG_THIS_PIXEL(inputImage);
		//	invert or flash a color?
		if (strobeMode == 0)	{
			srcPixel = (lastStateVector.r == 0.0) ? srcPixel : vec4(1.0-srcPixel.r, 1.0-srcPixel.g, 1.0-srcPixel.b, srcPixel.a);
		}
		else if (strobeMode == 1)	{
			srcPixel = (lastStateVector.r == 0.0) ? srcPixel : strobeColor;
		}
		gl_FragColor = srcPixel;
	}
}
