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
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": 0,
            "NAME": "intensity",
            "TYPE": "float"
        },
        {
            "DEFAULT": 6,
            "MAX": 15,
            "MIN": 1,
            "NAME": "radius",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "DESCRIPTION": "pass 0, 1x horiz sampling (3 wide total) to prevent data loss",
            "HEIGHT": "floor($HEIGHT/2.0)",
            "TARGET": "halfSize",
            "WIDTH": "floor($WIDTH/2.0)"
        },
        {
            "DESCRIPTION": "pass 1, vert sampling, use radius (size is being reduced, halve coords deltas when sampling)",
            "HEIGHT": "floor($HEIGHT/4.0)",
            "TARGET": "quarterSize",
            "WIDTH": "floor($WIDTH/4.0)"
        },
        {
            "DESCRIPTION": "pass 2, horiz sampling, use radius (size is being increased, use normal coords deltas)",
            "HEIGHT": "floor($HEIGHT/2.0)",
            "TARGET": "quarterSizePassB",
            "WIDTH": "floor($WIDTH/2.0)"
        },
        {
            "DESCRIPTION": "pass 3, last pass- samples the input image and the quarterSizePassB to generate the final image at the full original resolution"
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
	vec2		pixelWidth = 1.0/RENDERSIZE.xy;
	float		localBlurRadius;
	bool		vertFlag = false;
	
	if (PASSINDEX != 3)	{
		//	pass 0 and 2 are doing horizontal erosion
		if (PASSINDEX==1)
			vertFlag = true;
		//	the first pass should have a blur radius of 1.0 simply to prevent the loss of information while reducing resolution
		if (PASSINDEX==0)
			localBlurRadius = 1.0;
		//	other passes go by the blur radius!
		else
			localBlurRadius = float(int(radius));
		//	sample pixels as per the blur radius...
		for (float i=0.; i<=localBlurRadius; ++i)	{
			if (PASSINDEX==0)	{
				tmpCoord = vec2(clamp(isf_FragNormCoord.x+(i*pixelWidth.x), 0., 1.), isf_FragNormCoord.y);
				sampleColorRGB = IMG_NORM_PIXEL(inputImage, tmpCoord);
			}
			else if (PASSINDEX==1)	{
				tmpCoord = vec2(isf_FragNormCoord.x, clamp(isf_FragNormCoord.y+(i*pixelWidth.y/2.), 0., 1.));
				sampleColorRGB = IMG_NORM_PIXEL(halfSize, tmpCoord);
			}
			else if (PASSINDEX==2)	{
				tmpCoord = vec2(clamp(isf_FragNormCoord.x+(i*pixelWidth.x), 0., 1.), isf_FragNormCoord.y);
				sampleColorRGB = IMG_NORM_PIXEL(quarterSize, tmpCoord);
			}
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
				if (sampleColorHSV.b < maxLuma)	{
					maxLuma = sampleColorHSV.b;
					maxLumaRGBColor = sampleColorRGB;
				}
				//	do another sample for the negative coordinate
				if (PASSINDEX==0)	{
					tmpCoord = vec2(clamp(isf_FragNormCoord.x-(i*pixelWidth.x), 0., 1.), isf_FragNormCoord.y);
					sampleColorRGB = IMG_NORM_PIXEL(inputImage, tmpCoord);
				}
				else if (PASSINDEX==1)	{
					tmpCoord = vec2(isf_FragNormCoord.x, clamp(isf_FragNormCoord.y-(i*pixelWidth.y/2.), 0., 1.));
					sampleColorRGB = IMG_NORM_PIXEL(halfSize, tmpCoord);
				}
				else if (PASSINDEX==2)	{
					tmpCoord = vec2(clamp(isf_FragNormCoord.x-(i*pixelWidth.x), 0., 1.), isf_FragNormCoord.y);
					sampleColorRGB = IMG_NORM_PIXEL(quarterSize, tmpCoord);
				}
				sampleColorRGB = mix(maxLumaRGBColor, sampleColorRGB, 1.*intensity/i);
				sampleColorHSV.rgb = rgb2hsv(sampleColorRGB.rgb);
				if (sampleColorHSV.b < maxLuma)	{
					maxLuma = sampleColorHSV.b;
					maxLumaRGBColor = sampleColorRGB;
				}
			}
		}
		gl_FragColor = maxLumaRGBColor;
	}
	//	last pass
	else if (PASSINDEX==3)	{
		vec4		origPixel = IMG_NORM_PIXEL(inputImage,isf_FragNormCoord);
		vec4		erodedPixel = IMG_NORM_PIXEL(quarterSizePassB,isf_FragNormCoord);
		vec4		finalPixel = min(origPixel,erodedPixel);
		gl_FragColor = mix(origPixel,finalPixel, clamp(min(radius-1.0, intensity*10.0), 0., 1.));
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

