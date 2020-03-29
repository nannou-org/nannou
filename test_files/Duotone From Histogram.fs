/*
{
  "CATEGORIES" : [
    "Color Effect", "Histogram"
  ],
  "DESCRIPTION" : "Adjusts an image to use colors determined from a histogram",
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
		"NAME": "timeSmoothing",
		"TYPE": "float",
		"MIN": 0.0,
		"MAX": 1.0,
		"DEFAULT": 0.5
	},
	{
		"NAME": "minStart",
		"TYPE": "float",
		"MIN": 0.0,
		"MAX": 0.25,
		"DEFAULT": 0.0
	},
	{
		"NAME": "midRange",
		"TYPE": "float",
		"MIN": 0.0,
		"MAX": 0.5,
		"DEFAULT": 0.0
	},
	{
		"NAME": "averaging",
		"TYPE": "float",
		"MIN": 0.0,
		"MAX": 0.2,
		"DEFAULT": 0.0
	},
	{
		"NAME": "inThreshold",
		"TYPE": "float",
		"MIN": 0.0,
		"MAX": 0.1,
		"DEFAULT": 0.0
	},
	{
		"NAME": "outAutoThreshold",
		"TYPE": "bool",
		"DEFAULT": 1
	},
	{
		"NAME": "outThreshold",
		"TYPE": "float",
		"MIN": 0.0,
		"MAX": 1.0,
		"DEFAULT": 0.5
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

float brightness(vec3 c)	{
	return (c.r + c.g + c.b) / 3.0;
}

float minChannel (vec3 c)	{
	float	tmpVal = c.r;
	if (c.g > tmpVal)
		tmpVal = c.g;
	if (c.b > tmpVal)
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
		//	go through each pixel of sourceHistogram
		//		figure out the scan range based on isf_FragNormCoord and number of divisions
		//int			divisons = 2;
		int			rangeLength = 128 - int(midRange * 127.0);
		int			rangeStart = (isf_FragNormCoord.x < 0.5) ? int(minStart * 256.0) : 256 - rangeLength;
		vec4		currentMax = vec4(0.0);
		
		returnMe.a = 1.0;
		
		vec4		prev1 = vec4(0.0);
		vec4		prev2 = vec4(0.0);
		
		//	for the given range, find the index 
		for(int i=rangeStart;i<rangeStart + rangeLength;++i)	{
			vec4	histoStart = IMG_PIXEL(sourceHistogram,vec2(float(i),0.5));
			//	average this out with the 
			vec4	histo = (i <= rangeStart + 1) ? histoStart : (histoStart * (1.0 - averaging) + prev1 * averaging / 2.0 + prev2 * averaging / 2.0);
			
			if ((histo.r > inThreshold)&&(currentMax.r < histo.r))	{
				returnMe.r = float(i)/255.0;
				currentMax.r = histo.r;
			}
			if ((histo.g > inThreshold)&&(currentMax.g < histo.g))	{
				returnMe.g = float(i)/255.0;
				currentMax.g = histo.g;
			}
			if ((histo.b > inThreshold)&&(currentMax.b < histo.b))	{
				returnMe.b = float(i)/255.0;
				currentMax.b = histo.b;
			}
			prev2 = prev1;
			prev1 = histoStart;
		}
	}
	else if (PASSINDEX==1)	{
		vec4		freshPixel = IMG_PIXEL(minmax,gl_FragCoord.xy);
		vec4		stalePixel = IMG_PIXEL(smoothedminmax,gl_FragCoord.xy);
		returnMe = mix(freshPixel,stalePixel,timeSmoothing);
	}
	else if (PASSINDEX==2)	{
		returnMe = IMG_THIS_PIXEL(inputImage);
		vec4	minColor = IMG_PIXEL(smoothedminmax,vec2(0.5,0.5));
		vec4	maxColor = IMG_PIXEL(smoothedminmax,vec2(1.5,0.5));
		float	bVal = brightness(returnMe.rgb);
		float	outThresh = (outAutoThreshold) ? mix(minChannel(minColor.rgb),maxChannel(maxColor.rgb),0.5) : outThreshold;
		
		if (bVal < outThresh)	{
			returnMe.rgb = minColor.rgb;
		}
		else	{
			returnMe.rgb = maxColor.rgb;
		}
	}
	
	gl_FragColor = returnMe;
}
