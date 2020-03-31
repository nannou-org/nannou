/*{
    "CATEGORIES": [
        "Blur"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "intensity",
            "TYPE": "float"
        },
        {
            "DEFAULT": 2,
            "MAX": 15,
            "MIN": 1,
            "NAME": "bleedRadius",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "TARGET": "firstPass"
        },
        {
        }
    ]
}
*/



vec3 rgb2hsv(vec3 c);

void main()
{
	
	//	generally speaking, sample a bunch of pixels, for each sample convert color val to HSV, if luma is greater than max luma, that's the new color
	vec2		tmpCoord;
	float		maxLuma;
	vec4		maxLumaRGBColor = vec4(0.0);
	vec4		sampleColorRGB;
	vec4		sampleColorHSV;
	
	if (PASSINDEX==0)	{
		//	5 samples on either side + the source pixel = 11 total samples
		for (float i=0.; i<=float(int(bleedRadius)); ++i)	{
			tmpCoord = vec2(clamp(gl_FragCoord.x+i,0.,RENDERSIZE.x), gl_FragCoord.y);
			sampleColorRGB = IMG_PIXEL(inputImage, tmpCoord);
			//	if this is the first sample for this fragment, don't bother comparing- just set the max luma stuff
			if (i == 0.)	{
				maxLuma = rgb2hsv(sampleColorRGB.rgb).b;
				maxLumaRGBColor = sampleColorRGB;
			}
			//	else this isn't the first sample...
			else	{
				//	compare, determine if it's the max luma
				sampleColorRGB = mix(maxLumaRGBColor, sampleColorRGB, 1.*intensity/i);
				sampleColorHSV.rgb = rgb2hsv(sampleColorRGB.rgb);
				if (sampleColorHSV.b > maxLuma)	{
					maxLuma = sampleColorHSV.b;
					maxLumaRGBColor = sampleColorRGB;
				}
				//	do another sample for the negative coordinate
				tmpCoord = vec2(clamp(gl_FragCoord.x-i,0.,RENDERSIZE.x), gl_FragCoord.y);
				sampleColorRGB = IMG_PIXEL(inputImage, tmpCoord);
				sampleColorRGB = mix(maxLumaRGBColor, sampleColorRGB, 1.*intensity/i);
				sampleColorHSV.rgb = rgb2hsv(sampleColorRGB.rgb);
				if (sampleColorHSV.b > maxLuma)	{
					maxLuma = sampleColorHSV.b;
					maxLumaRGBColor = sampleColorRGB;
				}
			}
		}
		gl_FragColor = maxLumaRGBColor;
		
	}
	else if (PASSINDEX==1)	{
		//	5 samples up and down + the source pixel = 11 total samples
		for (float i=0.; i<=float(int(bleedRadius)); ++i)	{
			tmpCoord = vec2(gl_FragCoord.x, clamp(gl_FragCoord.y+i,0.,RENDERSIZE.y));
			sampleColorRGB = IMG_PIXEL(firstPass, tmpCoord);
			//	if this is the first sample for this fragment, don't bother comparing- just set the max luma stuff
			if (i == 0.)	{
				maxLuma = rgb2hsv(sampleColorRGB.rgb).b;
				maxLumaRGBColor = sampleColorRGB;
			}
			//	else this isn't the first sample...
			else	{
				//	compare, determine if it's the max luma
				sampleColorRGB = mix(maxLumaRGBColor, sampleColorRGB, 1.*intensity/i);
				sampleColorHSV.rgb = rgb2hsv(sampleColorRGB.rgb);
				if (sampleColorHSV.b > maxLuma)	{
					maxLuma = sampleColorHSV.b;
					maxLumaRGBColor = sampleColorRGB;
				}
				//	do another sample for the negative coordinate
				tmpCoord = vec2(gl_FragCoord.x, clamp(gl_FragCoord.y-i,0.,RENDERSIZE.y));
				sampleColorRGB = IMG_PIXEL(firstPass, tmpCoord);
				sampleColorRGB = mix(maxLumaRGBColor, sampleColorRGB, 1.*intensity/i);
				sampleColorHSV.rgb = rgb2hsv(sampleColorRGB.rgb);
				if (sampleColorHSV.b > maxLuma)	{
					maxLuma = sampleColorHSV.b;
					maxLumaRGBColor = sampleColorRGB;
				}
			}
		}
		gl_FragColor = maxLumaRGBColor;
		
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

