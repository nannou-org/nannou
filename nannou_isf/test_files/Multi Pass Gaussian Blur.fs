/*{
	"CREDIT": "original implementation as v002.blur in QC by anton marini and tom butterworth, ported by zoidberg",
	"ISFVSN": "2",
	"DESCRIPTION": "Performs a deep blur",
	"CATEGORIES": [
		"Blur"
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
			"MAX": 24.0,
			"DEFAULT": 24.0
		}
	],
	"PASSES": [
		{
			"TARGET": "halfSizeBaseRender",
			"WIDTH": "floor($WIDTH/3.0)",
			"HEIGHT": "floor($HEIGHT/3.0)",
			"DESCRIPTION": "Pass 0"
		},
		{
			"TARGET": "quarterSizeBaseRender",
			"WIDTH": "floor($WIDTH/6.0)",
			"HEIGHT": "floor($HEIGHT/6.0)",
			"DESCRIPTION": "Pass 1"
		},
		{
			"TARGET": "eighthSizeBaseRender",
			"WIDTH": "floor($WIDTH/12.0)",
			"HEIGHT": "floor($HEIGHT/12.0)",
			"DESCRIPTION": "Pass 2"
		},
		{
			"TARGET": "eighthGaussA",
			"WIDTH": "floor($WIDTH/12.0)",
			"HEIGHT": "floor($HEIGHT/12.0)",
			"DESCRIPTION": "Pass 3"
		},
		{
			"TARGET": "eighthGaussB",
			"WIDTH": "floor($WIDTH/12.0)",
			"HEIGHT": "floor($HEIGHT/12.0)",
			"DESCRIPTION": "Pass 4"
		},
		{
			"TARGET": "quarterGaussA",
			"WIDTH": "floor($WIDTH/6.0)",
			"HEIGHT": "floor($HEIGHT/6.0)",
			"DESCRIPTION": "Pass 5"
		},
		{
			"TARGET": "quarterGaussB",
			"WIDTH": "floor($WIDTH/6.0)",
			"HEIGHT": "floor($HEIGHT/6.0)",
			"DESCRIPTION": "Pass 6"
		},
		{
			"TARGET": "halfGaussA",
			"WIDTH": "floor($WIDTH/3.0)",
			"HEIGHT": "floor($HEIGHT/3.0)",
			"DESCRIPTION": "Pass 7"
		},
		{
			"TARGET": "halfGaussB",
			"WIDTH": "floor($WIDTH/3.0)",
			"HEIGHT": "floor($HEIGHT/3.0)",
			"DESCRIPTION": "Pass 8"
		},
		{
			"TARGET": "fullGaussA",
			"DESCRIPTION": "Pass 9"
		},
		{
			"TARGET": "fullGaussB",
			"DESCRIPTION": "Pass 10"
		}
	]
}*/


/*
																			eighth
													quarter					0	1	2	3	4	5		"blurRadius" (different resolutions have different blur radiuses based on the "blurAmount" and its derived "blurLevel")
							half					0	1	2	3	4	5								"blurRadius"
	normal					0	1	2	3	4	5														"blurRadius"
	0	1	2	3	4	5	5																			"blurRadius"
	0						6						12						18						24	"blurAmount" (attrib)
				0						1						2						3				"blurLevel" (local var)
*/

#if __VERSION__ <= 120
varying vec2		texOffsets[5];
#else
in vec2		texOffsets[5];
#endif


void main() {
	int			blurLevel = int(floor(blurAmount/6.0));
	float		blurLevelModulus = mod(blurAmount, 6.0);
	//	first three passes are just copying the input image into the buffer at varying sizes
	if (PASSINDEX==0)	{
		gl_FragColor = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord);
	}
	else if (PASSINDEX==1)	{
		gl_FragColor = IMG_NORM_PIXEL(halfSizeBaseRender, isf_FragNormCoord);
	}
	else if (PASSINDEX==2)	{
		gl_FragColor = IMG_NORM_PIXEL(quarterSizeBaseRender, isf_FragNormCoord);
	}
	//	start reading from the previous stage- each two passes completes a gaussian blur, then 
	//	we increase the resolution & blur (the lower-res blurred image from the previous pass) again...
	else if (PASSINDEX == 3)	{
		vec4		sample0 = IMG_NORM_PIXEL(eighthSizeBaseRender,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(eighthSizeBaseRender,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(eighthSizeBaseRender,texOffsets[2]);
		vec4		sample3 = IMG_NORM_PIXEL(eighthSizeBaseRender,texOffsets[3]);
		vec4		sample4 = IMG_NORM_PIXEL(eighthSizeBaseRender,texOffsets[4]);
		//gl_FragColor = vec4((sample0 + sample1 + sample2).rgb / (3.0), 1.0);
		gl_FragColor = vec4((sample0 + sample1 + sample2 + sample3 + sample4).rgba / (5.0));
	}
	else if (PASSINDEX == 4)	{
		vec4		sample0 = IMG_NORM_PIXEL(eighthGaussA,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(eighthGaussA,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(eighthGaussA,texOffsets[2]);
		vec4		sample3 = IMG_NORM_PIXEL(eighthGaussA,texOffsets[3]);
		vec4		sample4 = IMG_NORM_PIXEL(eighthGaussA,texOffsets[4]);
		//gl_FragColor = vec4((sample0 + sample1 + sample2).rgb / (3.0), 1.0);
		gl_FragColor = vec4((sample0 + sample1 + sample2 + sample3 + sample4).rgba / (5.0));
	}
	//	...writes into the quarter-size
	else if (PASSINDEX == 5)	{
		vec4		sample0 = IMG_NORM_PIXEL(eighthGaussB,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(eighthGaussB,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(eighthGaussB,texOffsets[2]);
		gl_FragColor =  vec4((sample0 + sample1 + sample2).rgba / (3.0));
	}
	else if (PASSINDEX == 6)	{
		vec4		sample0 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[2]);
		gl_FragColor =  vec4((sample0 + sample1 + sample2).rgba / (3.0));
	}
	//	...writes into the half-size
	else if (PASSINDEX == 7)	{
		vec4		sample0 = IMG_NORM_PIXEL(quarterGaussB,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(quarterGaussB,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(quarterGaussB,texOffsets[2]);
		gl_FragColor =  vec4((sample0 + sample1 + sample2).rgba / (3.0));
	}
	else if (PASSINDEX == 8)	{
		vec4		sample0 = IMG_NORM_PIXEL(halfGaussA,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(halfGaussA,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(halfGaussA,texOffsets[2]);
		gl_FragColor =  vec4((sample0 + sample1 + sample2).rgba / (3.0));
	}
	//	...writes into the full-size
	else if (PASSINDEX == 9)	{
		vec4		sample0 = IMG_NORM_PIXEL(halfGaussB,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(halfGaussB,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(halfGaussB,texOffsets[2]);
		gl_FragColor =  vec4((sample0 + sample1 + sample2).rgba / (3.0));
	}
	else if (PASSINDEX == 10)	{
		//	this is the last pass- calculate the blurred image as i have in previous passes, then mix it in with the full-size input image using the blur amount so i get a smooth transition into the blur at low blur levels
		vec4		sample0 = IMG_NORM_PIXEL(fullGaussA,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(fullGaussA,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(fullGaussA,texOffsets[2]);
		vec4		blurredImg =  vec4((sample0 + sample1 + sample2).rgba / (3.0));
		if (blurLevel == 0)
			gl_FragColor = mix(IMG_NORM_PIXEL(inputImage,isf_FragNormCoord), blurredImg, (blurLevelModulus/6.0));
		else
			gl_FragColor = blurredImg;
	}
	
}
