/*{
	"DESCRIPTION": "Masks an image based on the amount of motion",
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Masking"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"LABEL": "Threshold",
			"NAME": "threshold",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.25
		},
		{
			"LABEL": "Persistence",
			"NAME": "persistence",
			"TYPE": "float",
			"MIN": 0.9,
			"MAX": 1.0,
			"DEFAULT": 0.95
		},
		{
			"LABEL": "Reset BG",
			"NAME": "updateBackground",
			"TYPE": "event"
		},
		{
			"LABEL": "Dilate",
			"NAME": "dilate",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 6.0,
			"DEFAULT": 0.0
		},
		{
			"LABEL": "Blur",
			"NAME": "blur",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 6.0,
			"DEFAULT": 0.0
		},
		{
			"LABEL": "Output Mask",
			"NAME": "showAlpha",
			"TYPE": "bool",
			"DEFAULT": false
		},
		{
			"LABEL": "Hard Cutoff",
			"NAME": "applyCutOff",
			"TYPE": "bool",
			"DEFAULT": true
		}
	],
	"PASSES": [
		{
			"TARGET":"freezeBuffer",
			"PERSISTENT": true
		},
		{
			"TARGET":"mask"
		},
		{
			"TARGET": "horizDilate"
		},
		{
			"TARGET": "vertDilate"
		},
		{
			"TARGET": "horizBlur"
		},
		{
			"TARGET": "vertBlur"
		},
		{
			
		}
	]
	
}*/

//	Essentially the same as a feedback buffer where you can only set it to a mix of 1.0

void main()
{
	vec2		tmpCoord;
	float		maxAlpha;
	vec4		sampleColorRGB;
	vec4		src_rgb;
	
	if (PASSINDEX==0)	{
		vec4	srcPixel = IMG_THIS_PIXEL(inputImage);
		vec4	oldPixel = IMG_THIS_PIXEL(freezeBuffer);
		gl_FragColor = (updateBackground) ? srcPixel : mix(srcPixel,oldPixel,persistence);
	}
	else if (PASSINDEX==1)	{
		vec4	srcPixel = IMG_THIS_PIXEL(inputImage);
		vec4	oldPixel = IMG_THIS_PIXEL(freezeBuffer);
		vec4	resultPixel = vec4(0.0);
		float	difference = abs(srcPixel.r - oldPixel.r) + abs(srcPixel.g - oldPixel.g) + abs(srcPixel.b - oldPixel.b);
		resultPixel = vec4(difference);
		gl_FragColor = resultPixel;	
	}
	else if (PASSINDEX==2)	{
		//	'dilate' samples on either side + the source pixel = 2*dilate+1 total samples
		for (float i=0.; i<=dilate; ++i)	{
			tmpCoord = vec2(clamp(gl_FragCoord.x+i,0.,RENDERSIZE.x), gl_FragCoord.y);
			sampleColorRGB = IMG_PIXEL(mask, tmpCoord);
			//	if this is the first sample for this fragment, don't bother comparing- just set the max luma stuff
			if (i == 0.)	{
				maxAlpha = sampleColorRGB.a;
				src_rgb = sampleColorRGB;
			}
			//	else this isn't the first sample...
			else	{
				//	compare, determine if it's the max luma
				if (sampleColorRGB.a < maxAlpha)	{
					maxAlpha = sampleColorRGB.a;
				}
				//	do another sample for the negative coordinate
				tmpCoord = vec2(clamp(gl_FragCoord.x-i,0.,RENDERSIZE.x), gl_FragCoord.y);
				sampleColorRGB = IMG_PIXEL(mask, tmpCoord);
				if (sampleColorRGB.a < maxAlpha)	{
					maxAlpha = sampleColorRGB.a;
				}
			}
		}
		gl_FragColor = vec4(src_rgb.rgb, maxAlpha);
	}
	else if (PASSINDEX==3)	{
		//	'dilate' samples up and down + the source pixel = 2*dilate+1 total samples
		for (float i=0.; i<=float(int(dilate)); ++i)	{
			tmpCoord = vec2(gl_FragCoord.x, clamp(gl_FragCoord.y+i,0.,RENDERSIZE.y));
			sampleColorRGB = IMG_PIXEL(horizDilate, tmpCoord);
			//	if this is the first sample for this fragment, don't bother comparing- just set the max luma stuff
			if (i == 0.)	{
				maxAlpha = sampleColorRGB.a;
				src_rgb = sampleColorRGB;
			}
			//	else this isn't the first sample...
			else	{
				//	compare, determine if it's the max luma
				if (sampleColorRGB.a < maxAlpha)	{
					maxAlpha = sampleColorRGB.a;
				}
				//	do another sample for the negative coordinate
				tmpCoord = vec2(gl_FragCoord.x, clamp(gl_FragCoord.y-i,0.,RENDERSIZE.y));
				sampleColorRGB = IMG_PIXEL(horizDilate, tmpCoord);
				if (sampleColorRGB.a < maxAlpha)	{
					maxAlpha = sampleColorRGB.a;
				}
			}
		}
		gl_FragColor = vec4(src_rgb.rgb, maxAlpha);
	}
	else if (PASSINDEX==4)	{
		float		tmpRadius = float(int(blur));
		vec4		tmpColor;
		vec4		srcColor;
		float		totalAlpha = 0.0;
		for (float i=-1.*tmpRadius; i<=tmpRadius; ++i)	{
			tmpColor = IMG_PIXEL(vertDilate, gl_FragCoord.xy+vec2(i,0.0));
			totalAlpha += tmpColor.a;
			if (i==0.0)
				srcColor = tmpColor;
		}
		totalAlpha /= ((tmpRadius*2.)+1.);
		gl_FragColor = vec4(srcColor.r, srcColor.g, srcColor.b, totalAlpha);
	}
	else if (PASSINDEX==5)	{
		float		tmpRadius = float(int(blur));
		vec4		tmpColor;
		vec4		srcColor;
		float		totalAlpha = 0.0;
		for (float i=-1.*tmpRadius; i<=tmpRadius; ++i)	{
			tmpColor = IMG_PIXEL(horizBlur, gl_FragCoord.xy+vec2(0.0,i));
			totalAlpha += tmpColor.a;
			if (i==0.0)
				srcColor = tmpColor;
		}
		totalAlpha /= ((tmpRadius*2.)+1.);
		
		gl_FragColor = vec4(srcColor.r, srcColor.g, srcColor.b, totalAlpha);
	}
	else if (PASSINDEX==6)	{
		vec4	srcPixel = IMG_THIS_PIXEL(inputImage);
		vec4	oldPixel = IMG_THIS_PIXEL(vertBlur);
		vec4	resultPixel = vec4(0.0);
		float	difference = oldPixel.a;
		if (difference > threshold)	{
			if (showAlpha)
				resultPixel = vec4(1.0,1.0,1.0,1.0);
			else 
				resultPixel = srcPixel;
			
			if (!applyCutOff)
				resultPixel = resultPixel * difference;
		}
		gl_FragColor = resultPixel;
	}

}
