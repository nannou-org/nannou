/*
{
  "CATEGORIES" : [
    "Glitch"
  ],
  "DESCRIPTION" : "Keeps an accumulation of difference since a key frame",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "updateKeyFrame",
      "TYPE" : "bool",
      "DEFAULT" : 0,
      "LABEL" : "Update Key Frame"
    },
    {
      "NAME" : "adaptRate",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0,
      "LABEL" : "Adapt Rate",
      "MIN" : 0
    },
    {
      "NAME" : "numColors",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 1,
      "LABEL" : "Color Quality",
      "MIN" : 0
    },
    {
      "NAME" : "buffQuality",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 1.0,
      "LABEL" : "Buffer Quality",
      "MIN" : 0
    }
  ],
  "PASSES" : [
    {
      "TARGET" : "keyFrame",
      "PERSISTENT" : true
    },
    {
      "TARGET" : "diffFrame",
      "PERSISTENT" : true,
      "WIDTH" : "max(8.0,floor($WIDTH*$buffQuality))",
      "HEIGHT" : "max(8.0,floor($HEIGHT*$buffQuality))"
    },
    {
      "TARGET" : "lastFrame",
      "PERSISTENT" : true
    },
    {

    }
  ],
  "CREDIT" : "by zoidberg"
}
*/

vec3 cround (vec3 r)	{
	vec3 returnMe = r;
	returnMe.r = (fract(returnMe.r) < 0.5) ? floor(returnMe.r) : ceil(returnMe.r);
	returnMe.g = (fract(returnMe.g) < 0.5) ? floor(returnMe.g) : ceil(returnMe.g);
	returnMe.b = (fract(returnMe.b) < 0.5) ? floor(returnMe.b) : ceil(returnMe.b);	
	return returnMe;	
}

void main()
{
	//	on the first pass, set the key frame if needed
	
	if (PASSINDEX==0)	{
		
		bool		doUpdateKeyFrame = ((updateKeyFrame)||(FRAMEINDEX<5)) ? true : false;
		vec4		result = (doUpdateKeyFrame) ? IMG_THIS_NORM_PIXEL(inputImage) : IMG_THIS_NORM_PIXEL(keyFrame);
		if ((doUpdateKeyFrame)&&(numColors < 1.0))	{
			float		scaledColors = floor(pow(2.0,numColors * 8.0));
			scaledColors = (scaledColors < 2.0) ? 2.0 : scaledColors;
 			result.rgb = result.rgb * scaledColors;
 			result.rgb = cround(result.rgb);
 			result.rgb = result.rgb / (scaledColors);
		}
		gl_FragColor = result;
		
	}
	//	on the second pass, compare lastFrame to inputImage and add that amount to diffFrame
	else if (PASSINDEX==1)	{
		bool		doUpdateKeyFrame = ((updateKeyFrame)||(FRAMEINDEX<5)) ? true : false;
		vec4		freshPixel = IMG_THIS_NORM_PIXEL(inputImage);
		vec4		stalePixel = (doUpdateKeyFrame) ? IMG_THIS_NORM_PIXEL(keyFrame) : IMG_THIS_NORM_PIXEL(lastFrame);
		vec4		diffPixel = (doUpdateKeyFrame) ? vec4(0.5) : IMG_THIS_NORM_PIXEL(diffFrame);
		diffPixel.rgb = 2.0 * diffPixel.rgb - 1.0;
		vec4		result = diffPixel + freshPixel - stalePixel;
		result = (result + 1.0) / 2.0;
		if (numColors < 1.0)	{
			float		scaledColors = floor(pow(2.0,numColors * 8.0));
			scaledColors = (scaledColors < 2.0) ? 2.0 : scaledColors;
 			result.rgb = result.rgb * scaledColors;
 			result.rgb = cround(result.rgb);
 			result.rgb = result.rgb / (scaledColors);
		}
		
		gl_FragColor = result;
		
	}
	//	add the new diffFrame to keyFrame to get the new resulting frame
	else if (PASSINDEX==2)	{
		gl_FragColor = IMG_THIS_NORM_PIXEL(inputImage);
	}
	else	{
		vec4		keyFramePixel = IMG_THIS_NORM_PIXEL(keyFrame);
		vec4		diffPixel = IMG_THIS_NORM_PIXEL(diffFrame);
		diffPixel.rgb = (2.0 * diffPixel.rgb - 1.0);
		vec4		result = keyFramePixel + diffPixel;
		vec4		freshPixel = IMG_THIS_NORM_PIXEL(inputImage);
		result = mix(result,freshPixel,adaptRate);
		gl_FragColor = result;
	}
}
