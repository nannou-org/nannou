/*{
    "CATEGORIES": [
        "Blur",
        "Film"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 12,
            "MAX": 12,
            "MIN": 0,
            "NAME": "blurAmount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "intensity",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "DESCRIPTION": "Pass 0",
            "HEIGHT": "floor($HEIGHT/2.0)",
            "TARGET": "halfSizeBaseRender",
            "WIDTH": "floor($WIDTH/2.0)"
        },
        {
            "DESCRIPTION": "Pass 1",
            "HEIGHT": "floor($HEIGHT/4.0)",
            "TARGET": "quarterSizeBaseRender",
            "WIDTH": "floor($WIDTH/4.0)"
        },
        {
            "DESCRIPTION": "Pass 2",
            "HEIGHT": "floor($HEIGHT/8.0)",
            "TARGET": "eighthSizeBaseRender",
            "WIDTH": "floor($WIDTH/8.0)"
        },
        {
            "DESCRIPTION": "Pass 3",
            "HEIGHT": "floor($HEIGHT/4.0)",
            "TARGET": "quarterGaussA",
            "WIDTH": "floor($WIDTH/4.0)"
        },
        {
            "DESCRIPTION": "Pass 4",
            "HEIGHT": "floor($HEIGHT/4.0)",
            "TARGET": "quarterGaussB",
            "WIDTH": "floor($WIDTH/4.0)"
        },
        {
            "DESCRIPTION": "Pass 5",
            "TARGET": "fullGaussA"
        },
        {
            "DESCRIPTION": "Pass 6",
            "TARGET": "fullGaussB"
        }
    ]
}
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
		gl_FragColor = vec4((sample0 + sample1 + sample2 + sample3 + sample4).rgb / (5.0), 1.0);
	}
	else if (PASSINDEX == 4)	{
		vec4		sample0 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[2]);
		vec4		sample3 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[3]);
		vec4		sample4 = IMG_NORM_PIXEL(quarterGaussA,texOffsets[4]);
		//gl_FragColor = vec4((sample0 + sample1 + sample2).rgb / (3.0), 1.0);
		gl_FragColor = vec4((sample0 + sample1 + sample2 + sample3 + sample4).rgb / (5.0), 1.0);
	}
	//	...writes into the full-size
	else if (PASSINDEX == 5)	{
		vec4		sample0 = IMG_NORM_PIXEL(quarterGaussB,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(quarterGaussB,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(quarterGaussB,texOffsets[2]);
		gl_FragColor =  vec4((sample0 + sample1 + sample2).rgb / (3.0), 1.0);
	}
	else if (PASSINDEX == 6)	{
		//	this is the last pass- calculate the blurred image as i have in previous passes, then mix it in with the full-size input image using the blur amount so i get a smooth transition into the blur at low blur levels
		vec4		sample0 = IMG_NORM_PIXEL(fullGaussA,texOffsets[0]);
		vec4		sample1 = IMG_NORM_PIXEL(fullGaussA,texOffsets[1]);
		vec4		sample2 = IMG_NORM_PIXEL(fullGaussA,texOffsets[2]);
		vec4		blurredImg =  vec4((sample0 + sample1 + sample2).rgb / (3.0), 1.0);
		vec4		originalImg = IMG_NORM_PIXEL(inputImage,isf_FragNormCoord);
		if (blurLevel == 0)
			blurredImg = mix(originalImg, blurredImg, (blurLevelModulus/6.0));
		
		gl_FragColor = min(mix(originalImg,blurredImg*0.9,intensity), originalImg);
	}
}