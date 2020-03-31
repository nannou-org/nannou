/*{
    "CATEGORIES": [
        "Masking",
        "Color Effect"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": [
                0.24,
                0.77,
                0.38,
                1
            ],
            "LABEL": "Key Color",
            "NAME": "mask_color",
            "TYPE": "color"
        },
        {
            "DEFAULT": false,
            "LABEL": "Show Alpha",
            "NAME": "showAlpha",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0,
            "NAME": "applyAlpha",
            "TYPE": "bool"
        },
        {
            "DEFAULT": false,
            "LABEL": "Hard Cutoff",
            "NAME": "applyCutOff",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Alpha Mode",
            "LABELS": [
                "Additive",
                "Multiply",
                "Replace"
            ],
            "NAME": "alphaMode",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2
            ]
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "Cutoff Thresh",
            "MAX": 1,
            "MIN": 0,
            "NAME": "cutoffThresh",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "HUE- Tol.",
            "MAX": 0.25,
            "MIN": 0,
            "NAME": "hueTol",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "LABEL": "HUE- Min. Bracket",
            "MAX": 0.5,
            "MIN": 0,
            "NAME": "hueMinBracket",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "LABEL": "HUE- Max Bracket",
            "MAX": 0.5,
            "MIN": 0,
            "NAME": "hueMaxBracket",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "SAT- Tol.",
            "MAX": 0.5,
            "MIN": 0,
            "NAME": "satTol",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "SAT- Min. Bracket",
            "MAX": 1,
            "MIN": 0,
            "NAME": "satMinBracket",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "SAT- Max Bracket",
            "MAX": 1,
            "MIN": 0,
            "NAME": "satMaxBracket",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "VAL- Tol.",
            "MAX": 0.5,
            "MIN": 0,
            "NAME": "valTol",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "VAL- Min. Bracket",
            "MAX": 1,
            "MIN": 0,
            "NAME": "valMinBracket",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "VAL- Max Bracket",
            "MAX": 1,
            "MIN": 0,
            "NAME": "valMaxBracket",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Dilate",
            "MAX": 6,
            "MIN": 0,
            "NAME": "dilate",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Blur",
            "MAX": 6,
            "MIN": 0,
            "NAME": "blur",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "TARGET": "alphaPass"
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
        }
    ]
}
*/



vec3 rgb2hsv(vec3 c);
vec3 hsv2rgb(vec3 c);

void main() {
	//	generally speaking, sample a bunch of pixels, for each sample convert color val to HSV, if luma is greater than max luma, that's the new color
	vec2		tmpCoord;
	float		maxAlpha;
	vec4		sampleColorRGB;
	vec4		src_rgb;
	
	if (PASSINDEX==0)	{
		//	the goal is to calculate a new float value for the alpha channel, and/or desaturate the color, if this pixel is "close" to the mask color
		//	get the src pixel's color (as RGB), convert to HSV values
		src_rgb = IMG_PIXEL(inputImage, gl_FragCoord.xy);
		vec3			src_hsl = rgb2hsv(src_rgb.rgb);
		//	convert the passed color (which is RGB) to HSV values
		vec3			color_hsl = rgb2hsv(mask_color.rgb);
		
		
/*			-minBracket	-tolerance	color_hsl	+tolerance	+maxBracket
				|			|			|			|			|			val being examined
		----------------------------------------------------------------
				|						|						|			alpha
				1.0						0.0						1.0									*/
		
		
		float			hueAlpha;
		float			satAlpha;
		float			valAlpha;
		//	use a 'for' loop to generate the hue/sat/val alpha values
		int			i = 0;
		float		source;
		float		color;
		float		tolerance;
		vec2		bracket;
		float		alpha;
		float		minVal;
		float		maxVal;
		for (i=0; i<3; ++i)	{
			source = src_hsl[i];
			color = color_hsl[i];
			if (i==0)	{
				tolerance = hueTol;
				bracket = vec2(hueMinBracket, hueMaxBracket);
			}
			else if (i==1)	{
				tolerance = satTol;
				bracket = vec2(satMinBracket, satMaxBracket);
			}
			else if (i==2)	{
				tolerance = valTol;
				bracket = vec2(valMinBracket, valMaxBracket);
			}
			
			//	if the source sat is < the mask color sat
			if (source < color)	{
				//	the minVal is the bracket and tolerance, the maxVal is the mask color
				minVal = color - tolerance - bracket.x;
				maxVal = color;
				//	as i go from minVal to maxVal, the alpha goes from 1 to 0
				alpha = 1.0 - smoothstep(minVal, maxVal, source);
			}
			//	else if the source sat is > the mask color sat
			else if (source > color)	{
				//	the minVal is the mask color, the maxVal is the tolerance and bracket
				minVal = color;
				maxVal = color + tolerance + bracket.y;
				//	as i go from minVal to maxVal, the alpha goes from 0 to 1
				alpha = smoothstep(minVal, maxVal, source);
			}
			//	else the source sat == the mask color sat, the alpha should be 0
			else
				alpha = 0.0;
			
			if (i==0)	{
				hueAlpha = alpha;
			}
			else if (i==1)	{
				satAlpha = alpha;
			}
			else if (i==2)	{
				valAlpha = alpha;
			}
		}
		
		
		//	calculate the alpha i'll be applying
		src_rgb.a = sqrt(max(hueAlpha,max(satAlpha, valAlpha)));
		
		//	if we're doing a hard cutoff on the alpha, set it to 0.0 if below it
		if ((applyCutOff) && (src_rgb.a < cutoffThresh))	{
			src_rgb.a = 0.0;
		}
		//	otherwise, desaturate the color if appropriate, use the desaturated color
		else if (hueAlpha < 1.0)	{
			//src_hsl.y = 0.0;
			src_hsl.y = mix(0.0, src_hsl.y, hueAlpha);
			src_rgb.rgb = hsv2rgb(src_hsl.xyz);
		}
		
		
		gl_FragColor = src_rgb;
		//gl_FragColor = vec4(src_rgb.a, src_rgb.a, src_rgb.a, 1.0);
	}
	
	else if (PASSINDEX==1)	{
		//	'dilate' samples on either side + the source pixel = 2*dilate+1 total samples
		for (float i=0.; i<=float(int(dilate)); ++i)	{
			tmpCoord = vec2(clamp(gl_FragCoord.x+i,0.,RENDERSIZE.x), gl_FragCoord.y);
			sampleColorRGB = IMG_PIXEL(alphaPass, tmpCoord);
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
				sampleColorRGB = IMG_PIXEL(alphaPass, tmpCoord);
				if (sampleColorRGB.a < maxAlpha)	{
					maxAlpha = sampleColorRGB.a;
				}
			}
		}
		gl_FragColor = vec4(src_rgb.rgb, maxAlpha);
	}
	else if (PASSINDEX==2)	{
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
	else if (PASSINDEX==3)	{
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
	else if (PASSINDEX==4)	{
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
		
		//	if multiplying the alpha (eg this is being used with multiple chroma mask fx)
		if (alphaMode == 0)	{
			vec4		originalColor = IMG_PIXEL(inputImage, gl_FragCoord.xy);
			totalAlpha = originalColor.a + totalAlpha;	
		}
		else if (alphaMode == 1)	{
			vec4		originalColor = IMG_PIXEL(inputImage, gl_FragCoord.xy);
			totalAlpha = originalColor.a * totalAlpha;
		}
		
		if (showAlpha)	{
			gl_FragColor = vec4(totalAlpha, totalAlpha, totalAlpha, 1.0);
		}
		else	{
			if (applyAlpha)
				gl_FragColor = vec4(srcColor.r*totalAlpha, srcColor.g*totalAlpha, srcColor.b*totalAlpha, 1.0);
			else
				gl_FragColor = vec4(srcColor.r, srcColor.g, srcColor.b, totalAlpha);
		}
	}
}




vec3 rgb2hsv(vec3 c)	{
	vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	//vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
	//vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
	vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
	vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);
	
	float d = q.x - min(q.w, q.y);
	float e = 1.0e-10;
	return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)	{
	vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
	vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
	return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}