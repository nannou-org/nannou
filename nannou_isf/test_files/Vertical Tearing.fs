/*
{
  "CATEGORIES" : [
    "Glitch"
  ],
  "DESCRIPTION" : "This introduces a vertical tearing effect similar to when GL VBL sync is off.",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "tearPosition",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.5,
      "MIN" : 0
    }
  ],
  "PASSES" : [
    {
      "TARGET" : "oldImage"
    },
    {
      "TARGET" : "newImage",
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
	//	write the previous buffer into here
	if (PASSINDEX == 0)	{
		gl_FragColor = IMG_NORM_PIXEL(newImage,isf_FragNormCoord.xy);
	}
	else if (PASSINDEX == 1)	{
		gl_FragColor = IMG_NORM_PIXEL(inputImage,isf_FragNormCoord.xy);
	}
	else if (PASSINDEX == 2)	{
		vec4		freshPixel = IMG_NORM_PIXEL(inputImage,isf_FragNormCoord.xy);
		vec4		stalePixel = IMG_NORM_PIXEL(oldImage,isf_FragNormCoord.xy);
		gl_FragColor = (isf_FragNormCoord.y > tearPosition) ? freshPixel : stalePixel;
	}
}
