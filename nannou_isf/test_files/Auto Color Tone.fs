/*{
    "CATEGORIES": [
        "Color Effect"
    ],
    "CREDIT": "by VIDVOX",
    "DESCRIPTION": "Creates variations on a base color using a given algorithm.",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Sample Mode",
            "LABELS": [
                "Base Color",
                "Pixel Follow",
                "Color Average"
            ],
            "NAME": "sampleMode",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2
            ]
        },
        {
            "DEFAULT": 1,
            "LABEL": "Color Mode",
            "LABELS": [
                "Basic Complementary",
                "Split Complementary",
                "Compound Complementary",
                "Spectrum",
                "Shades",
                "Analogous",
                "Compound Analogous"
            ],
            "NAME": "colorModeOverride",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2,
                3,
                4,
                5,
                6
            ]
        },
        {
            "DEFAULT": 7,
            "LABEL": "Color Count",
            "LABELS": [
                "2",
                "3",
                "4",
                "5",
                "6",
                "7",
                "8",
                "9",
                "10",
                "11",
                "12",
                "13",
                "14",
                "15",
                "16"
            ],
            "NAME": "colorCount",
            "TYPE": "long",
            "VALUES": [
                2,
                3,
                4,
                5,
                6,
                7,
                8,
                9,
                10,
                11,
                12,
                13,
                14,
                15,
                16
            ]
        },
        {
            "DEFAULT": [
                0.25,
                0.59,
                0.9,
                1
            ],
            "LABEL": "Base Color",
            "NAME": "baseColor",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0.5,
                0.5
            ],
            "LABEL": "Pixel Point",
            "MAX": [
                1,
                1
            ],
            "MIN": [
                0,
                0
            ],
            "NAME": "pixelFollowLocation",
            "TYPE": "point2D"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "HEIGHT": "$HEIGHT / 100.0",
            "TARGET": "bufferPassA",
            "WIDTH": "$WIDTH / 100.0"
        },
        {
            "HEIGHT": "1.0",
            "TARGET": "autoColorBuffer",
            "WIDTH": "1.0",
            "persistent": true
        },
        {
        }
    ]
}
*/




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

float gray(vec4 c)	{
	return (c.r + c.g + c.b) * c.a / 3.0;	
}

void main()
{
	if (PASSINDEX == 0)	{
		vec4 		inputColor = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord);
		gl_FragColor = inputColor;
	}
	else if (PASSINDEX == 1)	{
		vec4 		inputColor = IMG_NORM_PIXEL(bufferPassA, isf_FragNormCoord);
		vec4 		oldColor = IMG_NORM_PIXEL(autoColorBuffer, vec2(0.5,0.5));
		gl_FragColor = mix(inputColor, oldColor, 0.8);
	}
	else if (PASSINDEX == 2)	{
		vec4 		inputColor = IMG_THIS_PIXEL(inputImage);
		vec4		inColor = baseColor;
		float		index = floor(gray(inputColor) * float(colorCount));	
		//float		index = floor(isf_FragNormCoord.x * float(colorCount));
		float		variation = 0.3236;	//	1/5 the golden ratio
		int			colorMode = colorModeOverride;

		if (sampleMode == 0)	{
			inColor = baseColor;
			inColor.rgb = rgb2hsv(inColor.rgb);
		}
		else if (sampleMode == 1)	{
			inColor = IMG_NORM_PIXEL(inputImage, pixelFollowLocation);
			inColor.rgb = rgb2hsv(inColor.rgb);
		}
		else if (sampleMode == 2)	{
			inColor = IMG_NORM_PIXEL(autoColorBuffer, vec2(0.5,0.5));
			inColor.rgb = rgb2hsv(inColor.rgb);
			if (inColor.b < 0.1)	{
				inColor = inColor * 1.5;	
			}
		}
	
		vec4		outColor = inColor;
	
		//	Basic complimentary  saturation and brightness variations on two fixed 180 degree opposite hues
		if (colorMode == 0)	{
			if (mod(index, 2.0) >= 1.0)	{
				outColor.r = outColor.r + 0.5;
				outColor.r = outColor.r - floor(outColor.r);
			}
		
			outColor.g = outColor.g - variation * floor(index / 2.0);
		
			if (outColor.g < 0.1)	{
				outColor.g = outColor.g + variation * floor(index / 2.0);
				outColor.g = outColor.g - floor(outColor.g);
			}
		
			outColor.b = outColor.b - variation * floor(index / 4.0);
			if (outColor.b < 0.2)	{
				outColor.b = outColor.b + variation * floor(index / 4.0);
				outColor.b = outColor.b - floor(outColor.b);
			}
		}
		//	Split complimentary  saturation and brightness variations on a 3 fixed 120 degree hues
		else if (colorMode == 1)	{
			float divisor = 3.0;
			float ratio = 0.45;
			if (mod(index, 3.0) >= 2.0)	{
				outColor.r = outColor.r - ratio;
			}
			else if (mod(index, 3.0) >= 1.0)	{
				outColor.r = outColor.r + ratio;
			}
		
			//outColor.g = outColor.g + variation * floor(index / divisor);
		
			if (mod(index, 5.0) >= 3.0)	{
				outColor.g = outColor.g - variation;
				outColor.g = outColor.g - floor(outColor.g);
			}
			outColor.b = outColor.b - variation * floor(index / (divisor));
			if (outColor.b < 0.1)	{
				outColor.b = outColor.b + variation * floor(index / (divisor));
				outColor.b = outColor.b - floor(outColor.b);
			}
		}
		//	Compound complimentary  a combination of shades, complimentary and analogous colors with slight shifts
		else if (colorMode == 2)	{
			if (mod(index, 3.0) >= 2.0)	{
				outColor.r = outColor.r + 0.5;
				outColor.r = outColor.r + variation * index * 1.0 / float(colorCount - 1) / 4.0;
			}
			else	{
				outColor.r = outColor.r + variation * index * 1.0 / float(colorCount - 1);
			}
			outColor.r = outColor.r - floor(outColor.r);
		
		
			if (mod(index, 2.0) >= 1.0)	{
				outColor.g = outColor.g + index * variation / 2.0;
			}
			else if (mod(index, 3.0) >= 2.0)	{
				outColor.g = outColor.g - variation / 2.0;
			}
			else	{
				outColor.g = outColor.g - index * variation / float(colorCount - 1);
			}
			if (outColor.g > 1.0)	{
				outColor.g = outColor.g - floor(outColor.g);
			}
		}
		//	Spectrum  hue shifts based on number of colors with minor saturation shifts
		else if (colorMode == 3)	{
			outColor.r = outColor.r + index * 1.0 / float(colorCount);
			if (mod(index, 3.0) >= 2.0)	{
				outColor.g = outColor.g - variation / 2.0;
				outColor.g = outColor.g - floor(outColor.g);
			}
			else if (mod(index, 4.0) >= 3.0)	{
				outColor.g = outColor.g + variation / 2.0;
				//outColor.g = outColor.g - floor(outColor.g);
			}
		}
		//	Shades  saturation and brightness variations on a single fixed hue
		else if (colorMode == 4)	{
			if (mod(index, 2.0) >= 1.0)	{
				outColor.b = outColor.b - (index * variation) / float(colorCount-1);
			}
			else	{
				outColor.b = outColor.b + (index * variation) / float(colorCount-1);
				outColor.b = outColor.b - floor(outColor.b);
			}
			if (outColor.b < 0.075)	{
				outColor.b = 1.0 - outColor.b * variation;
			}
		
			if (mod(index, 3.0) >= 2.0)	{
				outColor.g = outColor.g - (index * variation) / 2.0;
			}
			else if (mod(index, 4.0) >= 3.0)	{
				outColor.g = outColor.g + (index * variation) / 2.0;
			}
		
			if ((outColor.g > 1.0) || (outColor.g < 0.05))	{
				outColor.g = outColor.g - floor(outColor.g);
			}
		}
		//	Analogous  small hue and saturation shifts 
		else if (colorMode == 5)	{

			outColor.r = outColor.r + variation * index * 1.0 / float(colorCount - 1); 		

			if (mod(index, 3.0) >= 1.0)	{
				outColor.g = outColor.g - variation / 2.0;
				if (outColor.g < 0.0)	{
					outColor.g = outColor.g + variation / 2.0;
				}
				if (outColor.g > 1.0)	{
					outColor.g = outColor.g - floor(outColor.g);
				}
			}
		}
		//	Compound Analogous  similar to analogous but with negative hue shifts
		else if (colorMode == 6)	{
			if (mod(index, 3.0) >= 1.0)	{
				outColor.r = outColor.r + variation * index * 1.0 / float(colorCount - 1);
			}
			else	{
				outColor.r = outColor.r - variation * index * 0.5 / float(colorCount - 1);	
			}
			if (mod(index, 3.0) >= 1.0)	{
				outColor.g = outColor.g - variation / 2.0;
				if (outColor.g < 0.0)	{
					outColor.g = outColor.g + variation;
				}
				if (outColor.g > 1.0)	{
					outColor.g = outColor.g - floor(outColor.g);
				}
			}
			if (mod(index, 4.0) >= 2.0)	{
				if (outColor.b < variation)	{
					outColor.b = outColor.b + variation;
				}				
			}
		}
	
		gl_FragColor = vec4(hsv2rgb(outColor.rgb), inColor.a);
	}
}
