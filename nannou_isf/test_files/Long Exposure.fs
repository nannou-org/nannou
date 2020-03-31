/*
{
  "CATEGORIES" : [
    "Color Effect", "Feedback", "Film"
  ],
  "DESCRIPTION" : "Bright objects burn in and optionally decay",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "absorptionRate",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.5,
      "MIN" : -1,
      "LABEL" : "Absorption Rate"
    },
    {
      "NAME" : "dischargeRate",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.25,
      "MIN" : 0,
      "LABEL" : "Discharge Rate"
    }
  ],
  "PASSES" : [
    {
      "TARGET" : "feedbackBuffer",
      "PERSISTENT" : true,
      "FLOAT" : true
    },
    {

    }
  ],
  "CREDIT" : "VIDVOX"
}
*/

void main()
{
	vec4		freshPixel = IMG_PIXEL(inputImage,gl_FragCoord.xy);
	vec4		stalePixel = IMG_PIXEL(feedbackBuffer,gl_FragCoord.xy);
	vec4		resultPixel = vec4(0.0);
	
	//	absorb and discharge
	if (PASSINDEX==0)	{
		//	start with the old pixel amount
		resultPixel = stalePixel;
		//	discharge from previous pass
		resultPixel *= (1.0 - dischargeRate);
		//	add this pass
		resultPixel += (freshPixel * absorptionRate);
		resultPixel = clamp(resultPixel,0.0,1.0);
	}
	//	composite 
	else if (PASSINDEX==1)	{
		resultPixel = freshPixel + stalePixel;
	}
	
	gl_FragColor = resultPixel;
}
