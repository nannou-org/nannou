/*
{
  "CATEGORIES" : [
    "Color Adjustment", "Histogram"
  ],
  "DESCRIPTION" : "",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "sourceHistogram",
      "TYPE" : "image"
    },
	{
		"DEFAULT": 0,
		"LABEL": "Normalization Mode",
		"LABELS": [
			"Average",
			"Darker",
			"Brighter",
			"RGB"
		],
		"NAME": "colorMode",
		"TYPE": "long",
		"VALUES": [
			0,
			1,
			2,
			3
		]
	},
	{
		"DEFAULT": 1,
		"LABEL": "Adjustment Type",
		"LABELS": [
			"Contrast",
			"Gamma"
		],
		"NAME": "colorAdjustment",
		"TYPE": "long",
		"VALUES": [
			0,
			1
		]
	},
    {
      "NAME" : "maxGain",
      "TYPE" : "float",
      "MAX" : 8,
      "DEFAULT" : 2,
      "MIN" : 1
    },
    {
      "NAME" : "threshold",
      "TYPE" : "float",
      "MAX" : 1.0,
      "DEFAULT" : 0.02,
      "MIN" : 0.0
    },
    {
      "NAME" : "timeSmoothing",
      "TYPE" : "float",
      "MAX" : 1.0,
      "DEFAULT" : 0.0,
      "MIN" : 0.0
    }
  ],
  "PASSES" : [
    {
      "WIDTH" : "2",
      "TARGET" : "minmax",
      "HEIGHT" : "1"
    },
    {
      "WIDTH" : "2",
      "TARGET" : "smoothedminmax",
      "HEIGHT" : "1",
      "PERSISTENT": true,
      "FLOAT": true
    },
    {

    }
  ],
  "CREDIT" : "VIDVOX"
}
*/

float minChannel (vec3 c)	{
	float	tmpVal = c.r;
	if (c.g < tmpVal)
		tmpVal = c.g;
	if (c.b < tmpVal)
		tmpVal = c.b;
	return tmpVal;
}

float maxChannel (vec3 c)	{
	float	tmpVal = c.r;
	if (c.g > tmpVal)
		tmpVal = c.g;
	if (c.b > tmpVal)
		tmpVal = c.b;
	return tmpVal;
}


void main()	{
	vec4		returnMe = vec4(0.0);
	if (PASSINDEX==0)	{
		//float		minOrMax = (isf_FragNormCoord.x < 0.5) ? 0.0 : 1.0;
		vec4		accum = vec4(0.0);
		//	go through each pixel of sourceHistogram
		//		find the darkest and brightest for each color channel
		if (isf_FragNormCoord.x < 0.5)	{
			returnMe.a = 1.0;
			for(int i=0;i<256;++i)	{
				vec4	histo = IMG_PIXEL(sourceHistogram,vec2(float(i),0.5));
				accum += histo;
				if (returnMe.r == 0.0)	{
					if (accum.r > threshold)	{
						returnMe.r = float(i)/255.0;
					}
				}
				if (returnMe.g == 0.0)	{
					if (accum.g > threshold)	{
						returnMe.g = float(i)/255.0;
					}
				}
				if (returnMe.b == 0.0)	{
					if (accum.b > threshold)	{
						returnMe.b = float(i)/255.0;
					}
				}
				if ((returnMe.r != 0.0)&&(returnMe.g != 0.0)&&(returnMe.b != 0.0))	{
					break;
				}
			}
		}
		else	{
			returnMe = vec4(1.0);
			for(int i=255;i>=0;--i)	{
				vec4	histo = IMG_PIXEL(sourceHistogram,vec2(float(i),0.5));
				accum += histo;
				if (returnMe.r == 1.0)	{
					if (accum.r > threshold)	{
						returnMe.r = float(i)/255.0;
					}
				}
				if (returnMe.g == 1.0)	{
					if (accum.g > threshold)	{
						returnMe.g = float(i)/255.0;
					}
				}
				if (returnMe.b == 1.0)	{
					if (accum.b > threshold)	{
						returnMe.b = float(i)/255.0;
					}
				}
				if ((returnMe.r != 1.0)&&(returnMe.g != 1.0)&&(returnMe.b != 1.0))	{
					break;
				}
			}
		}
	}
	else if (PASSINDEX==1)	{
		vec4		freshPixel = IMG_PIXEL(minmax,gl_FragCoord.xy);
		vec4		stalePixel = IMG_PIXEL(smoothedminmax,gl_FragCoord.xy);
		returnMe = mix(freshPixel,stalePixel,timeSmoothing);
		
		if (colorMode == 0)	{
			//	Average
			returnMe = vec4((returnMe.r + returnMe.g + returnMe.b) / 3.0);
		}
		else if (colorMode == 1)	{
			//	we want to subtract the biggest and maximize the spread
			//	find the brightest rgb channel
			float	tmpVal = maxChannel(returnMe.rgb);
			returnMe = vec4(tmpVal);
		}
		else if (colorMode == 2)	{
			//	we want to subtract the smallest and minimize the spread
			//	find the darkest rgb channel in min
			float	tmpVal = minChannel(returnMe.rgb);
			returnMe = vec4(tmpVal);
		}
	}
	else if (PASSINDEX==2)	{
		returnMe = IMG_THIS_PIXEL(inputImage);
		vec4	minVal = IMG_PIXEL(smoothedminmax,vec2(0.5,0.5));
		vec4	maxVal = IMG_PIXEL(smoothedminmax,vec2(1.5,0.5));
		
		if (colorAdjustment == 0)	{
			vec4	contrastVal = (colorMode == 1) ? (maxVal - minVal) : 1.0 / (maxVal - minVal);
		
			if (colorMode == 1)	{
				float	iMaxGain = 1.0 / maxGain;
				if (contrastVal.r < iMaxGain)
					contrastVal.r = iMaxGain;

				if (contrastVal.g < iMaxGain)
					contrastVal.g = iMaxGain;
		
				if (contrastVal.b < iMaxGain)
					contrastVal.b = iMaxGain;
			}
			else	{
				if (contrastVal.r > maxGain)
					contrastVal.r = maxGain;

				if (contrastVal.g > maxGain)
					contrastVal.g = maxGain;
		
				if (contrastVal.b > maxGain)
					contrastVal.b = maxGain;
			}
		
			returnMe.r = (returnMe.r - minVal.r) * contrastVal.r;
			returnMe.g = (returnMe.g - minVal.g) * contrastVal.g;
			returnMe.b = (returnMe.b - minVal.b) * contrastVal.b;
		}
		else if (colorAdjustment == 1)	{
			//	the input gamma range is 0.0-1.0 (normalized).  the actual gamma range i want to use is 0.0 - 5.0.
			//	however, actual gamma 0.0-1.0 is just as interesting as actual gamma 1.0-5.0, so we scale the normalized input to match...
			vec4	realGamma = (colorMode == 1) ? 1.0 / (maxVal - minVal) - 1.0 : 1.0 / (maxVal - minVal) - 0.5;
			
			if (realGamma.r > maxGain)
				realGamma.r = maxGain;

			if (realGamma.g > maxGain)
				realGamma.g = maxGain;
	
			if (realGamma.b > maxGain)
				realGamma.b = maxGain;
			
			float	iMaxGain = 1.0 / maxGain;
			if (realGamma.r < iMaxGain)
				realGamma.r = iMaxGain;

			if (realGamma.g < iMaxGain)
				realGamma.g = iMaxGain;
	
			if (realGamma.b < iMaxGain)
				realGamma.b = iMaxGain;
			
			returnMe.rgb = pow(returnMe.rgb, vec3(1.0/realGamma));
		}
	}
	
	gl_FragColor = returnMe;
}
