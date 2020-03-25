/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Stylize"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "blurAmount",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 12.0,
			"DEFAULT": 4.0
		},
		{
			"NAME": "intensity",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.5
		},
		{
			"NAME": "edgeIntensity",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 100.0,
			"DEFAULT": 15.0
		},
		{
			"NAME": "edgeThreshold",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "erodeIntensity",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.25
		},
		{
			"NAME": "erodeRadius",
			"TYPE": "float",
			"MIN": 1.0,
			"MAX": 15.0,
			"DEFAULT": 1.0
		}
	],
	"PASSES": [
		{
			"TARGET": "edges",
			"DESCRIPTION": "PASSINDEX is 0",
			"DESCRIPTION": "Pass 0"
		},
		{
			"TARGET": "halfSize",
			"WIDTH": "floor($WIDTH/2.0)",
			"HEIGHT": "floor($HEIGHT/2.0)",
			"DESCRIPTION": "pass 1, 1x horiz sampling (3 wide total) to prevent data loss"
		},
		{
			"TARGET": "quarterSizePassA",
			"WIDTH": "floor($WIDTH/4.0)",
			"HEIGHT": "floor($HEIGHT/4.0)",
			"DESCRIPTION": "pass 2, vert sampling, use erodeRadius (size is being reduced, halve coords deltas when sampling)"
		},
		{
			"TARGET": "quarterSizePassB",
			"WIDTH": "floor($WIDTH/4.0)",
			"HEIGHT": "floor($HEIGHT/4.0)",
			"DESCRIPTION": "pass 3, horiz sampling, use erodeRadius"
		},
		{
			"TARGET": "quarterGaussA",
			"WIDTH": "floor($WIDTH/4.0)",
			"HEIGHT": "floor($HEIGHT/4.0)",
			"DESCRIPTION": "Pass 4"
		},
		{
			"TARGET": "quarterGaussB",
			"WIDTH": "floor($WIDTH/4.0)",
			"HEIGHT": "floor($HEIGHT/4.0)",
			"DESCRIPTION": "Pass 5"
		},
		{
			"TARGET": "fullGaussA",
			"DESCRIPTION": "Pass 6"
		},
		{
			"TARGET": "fullGaussB",
			"DESCRIPTION": "Pass 7"
		}
	]
}*/

#if __VERSION__ <= 120
varying vec2		left_coord;
varying vec2		right_coord;
varying vec2		above_coord;
varying vec2		below_coord;

varying vec2		lefta_coord;
varying vec2		righta_coord;
varying vec2		leftb_coord;
varying vec2		rightb_coord;

varying vec2		texOffsets[5];
#else
in vec2		left_coord;
in vec2		right_coord;
in vec2		above_coord;
in vec2		below_coord;

in vec2		lefta_coord;
in vec2		righta_coord;
in vec2		leftb_coord;
in vec2		rightb_coord;

in vec2		texOffsets[5];
#endif

vec3 rgb2hsv(vec3 c);
float gray(vec4 n);



void main() {
	int			blurLevel = int(floor(blurAmount/6.0));
	float		blurLevelModulus = mod(blurAmount, 6.0);
	//	first pass draws edges
	if (PASSINDEX==0)	{
		vec4 color = IMG_THIS_PIXEL(inputImage);
		vec4 colorL = IMG_NORM_PIXEL(inputImage, left_coord);
		vec4 colorR = IMG_NORM_PIXEL(inputImage, right_coord);
		vec4 colorA = IMG_NORM_PIXEL(inputImage, above_coord);
		vec4 colorB = IMG_NORM_PIXEL(inputImage, below_coord);
	
		vec4 colorLA = IMG_NORM_PIXEL(inputImage, lefta_coord);
		vec4 colorRA = IMG_NORM_PIXEL(inputImage, righta_coord);
		vec4 colorLB = IMG_NORM_PIXEL(inputImage, leftb_coord);
		vec4 colorRB = IMG_NORM_PIXEL(inputImage, rightb_coord);
	
		float gx = (-1.0 * gray(colorLA)) + (-2.0 * gray(colorL)) + (-1.0 * gray(colorLB)) + (1.0 * gray(colorRA)) + (2.0 * gray(colorR)) + (1.0 * gray(colorRB));
		float gy = (1.0 * gray(colorLA)) + (2.0 * gray(colorA)) + (1.0 * gray(colorRA)) + (-1.0 * gray(colorRB)) + (-2.0 * gray(colorB)) + (-1.0 * gray(colorLB));
		
		float bright = pow(gx*gx + gy*gy,0.5);
		vec4 final = color * bright;
		
		//	if the brightness is below the threshold draw black
		if (bright < edgeThreshold)	{
			final = vec4(0.0, 0.0, 0.0, 1.0);
		}
		else	{
			final = final * edgeIntensity;
			final.a = 1.0;
		}
		
		gl_FragColor = final;
	}
	//	next three passes are doing an erode.  the first and third are doing horizontal sampling, the second is doing vert sampling
	else if (PASSINDEX==1 || PASSINDEX==2 || PASSINDEX==3)	{
		//	generally speaking, sample a bunch of pixels, for each sample convert color val to HSV, if luma is greater than max luma, that's the new color
		vec2		tmpCoord;
		float		maxLuma;
		vec4		maxLumaRGBColor = vec4(0.0);
		vec4		sampleColorRGB;
		vec4		sampleColorHSV;
		vec2		pixelWidth = 1.0/RENDERSIZE.xy;
		float		localBlurRadius;
		bool		vertFlag = false;
		
		//	pass 0 and 2 are doing horizontal erosion
		if (PASSINDEX==2)
			vertFlag = true;
		//	the first pass should have a blur radius of 1.0 simply to prevent the loss of information while reducing resolution
		if (PASSINDEX==1)
			localBlurRadius = 1.0;
		//	other passes go by the blur radius!
		else
			localBlurRadius = float(int(erodeRadius));
		//	sample pixels as per the blur radius...
		for (float i=0.; i<=localBlurRadius; ++i)	{
			if (PASSINDEX==1)	{
				tmpCoord = vec2(clamp(isf_FragNormCoord.x+(i*pixelWidth.x), 0., 1.), isf_FragNormCoord.y);
				sampleColorRGB = IMG_NORM_PIXEL(edges, tmpCoord);
			}
			else if (PASSINDEX==2)	{
				tmpCoord = vec2(isf_FragNormCoord.x, clamp(isf_FragNormCoord.y+(i*pixelWidth.y), 0., 1.));
				sampleColorRGB = IMG_NORM_PIXEL(halfSize, tmpCoord);
			}
			else if (PASSINDEX==3)	{
				tmpCoord = vec2(clamp(isf_FragNormCoord.x+(i*pixelWidth.x), 0., 1.), isf_FragNormCoord.y);
				sampleColorRGB = IMG_NORM_PIXEL(quarterSizePassA, tmpCoord);
			}
			//	if this is the first sample for this fragment, don't bother comparing- just set the max luma stuff
			if (i == 0.)	{
				maxLuma = rgb2hsv(sampleColorRGB.rgb).b;
				maxLumaRGBColor = sampleColorRGB;
			}
			//	else this isn't the first sample...
			else	{
				//	compare, determine if it's the max luma
				sampleColorRGB = mix(maxLumaRGBColor, sampleColorRGB, 1.*erodeIntensity/i);
				sampleColorHSV.rgb = rgb2hsv(sampleColorRGB.rgb);
				if (sampleColorHSV.b > maxLuma)	{
					maxLuma = sampleColorHSV.b;
					maxLumaRGBColor = sampleColorRGB;
				}
				//	do another sample for the negative coordinate
				if (PASSINDEX==1)	{
					tmpCoord = vec2(clamp(isf_FragNormCoord.x-(i*pixelWidth.x), 0., 1.), isf_FragNormCoord.y);
					sampleColorRGB = IMG_NORM_PIXEL(edges, tmpCoord);
				}
				else if (PASSINDEX==2)	{
					tmpCoord = vec2(isf_FragNormCoord.x, clamp(isf_FragNormCoord.y-(i*pixelWidth.y), 0., 1.));
					sampleColorRGB = IMG_NORM_PIXEL(halfSize, tmpCoord);
				}
				else if (PASSINDEX==3)	{
					tmpCoord = vec2(clamp(isf_FragNormCoord.x-(i*pixelWidth.x), 0., 1.), isf_FragNormCoord.y);
					sampleColorRGB = IMG_NORM_PIXEL(quarterSizePassA, tmpCoord);
				}
				sampleColorRGB = mix(maxLumaRGBColor, sampleColorRGB, 1.*erodeIntensity/i);
				sampleColorHSV.rgb = rgb2hsv(sampleColorRGB.rgb);
				if (sampleColorHSV.b > maxLuma)	{
					maxLuma = sampleColorHSV.b;
					maxLumaRGBColor = sampleColorRGB;
				}
			}
		}
		gl_FragColor = maxLumaRGBColor;
		
	}
	//	start reading from the previous stage- each two passes completes a gaussian blur, then 
	//	we increase the resolution & blur (the lower-res blurred image from the previous pass) again...
	else if (PASSINDEX == 4)	{
		vec4		sample0 = IMG_NORM_PIXEL(quarterSizePassB,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(quarterSizePassB,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(quarterSizePassB,texOffsets[2]);
		vec4		sample3 = IMG_NORM_PIXEL(quarterSizePassB,texOffsets[3]);
		vec4		sample4 = IMG_NORM_PIXEL(quarterSizePassB,texOffsets[4]);
		//gl_FragColor = vec4((sample0 + sample1 + sample2).rgb / (3.0), 1.0);
		gl_FragColor = vec4((sample0 + sample1 + sample2 + sample3 + sample4).rgb / (5.0), 1.0);
	}
	else if (PASSINDEX == 5)	{
		vec4		sample0 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[2]);
		vec4		sample3 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[3]);
		vec4		sample4 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[4]);
		//gl_FragColor = vec4((sample0 + sample1 + sample2).rgb / (3.0), 1.0);
		gl_FragColor = vec4((sample0 + sample1 + sample2 + sample3 + sample4).rgb / (5.0), 1.0);
	}
	//	...writes into the full-size
	else if (PASSINDEX == 6)	{
		vec4		sample0 = IMG_NORM_PIXEL(quarterGaussB,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(quarterGaussB,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(quarterGaussB,texOffsets[2]);
		gl_FragColor =  vec4((sample0 + sample1 + sample2).rgb / (3.0), 1.0);
	}
	else if (PASSINDEX == 7)	{
		//	this is the last pass- calculate the blurred image as i have in previous passes, then mix it in with the full-size input image using the blur amount so i get a smooth transition into the blur at low blur levels
		vec4		sample0 = IMG_NORM_PIXEL(fullGaussA,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(fullGaussA,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(fullGaussA,texOffsets[2]);
		vec4		blurredImg =  vec4((sample0 + sample1 + sample2).rgb / (3.0), 1.0);
		vec4		originalImg = IMG_NORM_PIXEL(edges,isf_FragNormCoord);
		if (blurLevel == 0)
			blurredImg = mix(originalImg, blurredImg, (blurLevelModulus/6.0));
		
		gl_FragColor = max(mix(originalImg,blurredImg*1.9,intensity), originalImg);
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
float gray(vec4 n)
{
	return (n.r + n.g + n.b)/3.0;
}
