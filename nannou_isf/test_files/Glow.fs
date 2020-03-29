/*{
    "CATEGORIES": [
        "Stylize"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "intensity",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "HEIGHT": "floor($HEIGHT*min((0.2),1.0))",
            "TARGET": "smallA",
            "WIDTH": "floor($WIDTH*min((0.2),1.0))"
        },
        {
            "HEIGHT": "floor($HEIGHT*min((0.2),1.0))",
            "TARGET": "smallB",
            "WIDTH": "floor($WIDTH*min((0.2),1.0))"
        },
        {
            "HEIGHT": "floor($HEIGHT*min((0.3),1.0))",
            "TARGET": "smallC",
            "WIDTH": "floor($WIDTH*min((0.3),1.0))"
        },
        {
            "HEIGHT": "floor($HEIGHT*min((0.3),1.0))",
            "TARGET": "smallD",
            "WIDTH": "floor($WIDTH*min((0.3),1.0))"
        },
        {
            "HEIGHT": "floor($HEIGHT*min((0.5),1.0))",
            "TARGET": "smallE",
            "WIDTH": "floor($WIDTH*min((0.5),1.0))"
        },
        {
            "HEIGHT": "floor($HEIGHT*min((0.5),1.0))",
            "TARGET": "smallF",
            "WIDTH": "floor($WIDTH*min((0.5),1.0))"
        },
        {
            "HEIGHT": "floor($HEIGHT*min((0.8),1.0))",
            "TARGET": "smallG",
            "WIDTH": "floor($WIDTH*min((0.8),1.0))"
        },
        {
            "HEIGHT": "floor($HEIGHT*min((0.8),1.0))",
            "TARGET": "smallH",
            "WIDTH": "floor($WIDTH*min((0.8),1.0))"
        },
        {
            "TARGET": "smallI"
        },
        {
        }
    ]
}
*/


//	original blur implementation as v002.blur in QC by anton marini and tom butterworth, ported by zoidberg

#if __VERSION__ <= 120
varying vec2		texOffsets[3];
#else
in vec2		texOffsets[3];
#endif


void main() {
	vec4		sample0;
	vec4		sample1;
	vec4		sample2;
	if (PASSINDEX == 0)	{
		sample0 = IMG_NORM_PIXEL(inputImage,texOffsets[0]);
		sample1 = IMG_NORM_PIXEL(inputImage,texOffsets[1]);
		sample2 = IMG_NORM_PIXEL(inputImage,texOffsets[2]);
	}
	else if (PASSINDEX == 1)	{
		sample0 = IMG_NORM_PIXEL(smallA,texOffsets[0]);
		sample1 = IMG_NORM_PIXEL(smallA,texOffsets[1]);
		sample2 = IMG_NORM_PIXEL(smallA,texOffsets[2]);
	}
	else if (PASSINDEX == 2)	{
		sample0 = IMG_NORM_PIXEL(smallB,texOffsets[0]);
		sample1 = IMG_NORM_PIXEL(smallB,texOffsets[1]);
		sample2 = IMG_NORM_PIXEL(smallB,texOffsets[2]);
	}
	else if (PASSINDEX == 3)	{
		sample0 = IMG_NORM_PIXEL(smallC,texOffsets[0]);
		sample1 = IMG_NORM_PIXEL(smallC,texOffsets[1]);
		sample2 = IMG_NORM_PIXEL(smallC,texOffsets[2]);
	}
	else if (PASSINDEX == 4)	{
		sample0 = IMG_NORM_PIXEL(smallD,texOffsets[0]);
		sample1 = IMG_NORM_PIXEL(smallD,texOffsets[1]);
		sample2 = IMG_NORM_PIXEL(smallD,texOffsets[2]);
	}
	else if (PASSINDEX == 5)	{
		sample0 = IMG_NORM_PIXEL(smallE,texOffsets[0]);
		sample1 = IMG_NORM_PIXEL(smallE,texOffsets[1]);
		sample2 = IMG_NORM_PIXEL(smallE,texOffsets[2]);
	}
	else if (PASSINDEX == 6)	{
		sample0 = IMG_NORM_PIXEL(smallF,texOffsets[0]);
		sample1 = IMG_NORM_PIXEL(smallF,texOffsets[1]);
		sample2 = IMG_NORM_PIXEL(smallF,texOffsets[2]);
	}
	else if (PASSINDEX == 7)	{
		sample0 = IMG_NORM_PIXEL(smallG,texOffsets[0]);
		sample1 = IMG_NORM_PIXEL(smallG,texOffsets[1]);
		sample2 = IMG_NORM_PIXEL(smallG,texOffsets[2]);
	}
	else if (PASSINDEX == 8)	{
		sample0 = IMG_NORM_PIXEL(smallH,texOffsets[0]);
		sample1 = IMG_NORM_PIXEL(smallH,texOffsets[1]);
		sample2 = IMG_NORM_PIXEL(smallH,texOffsets[2]);
	}
	else if (PASSINDEX == 9)	{
		sample0 = IMG_NORM_PIXEL(smallI,texOffsets[0]);
		sample1 = IMG_NORM_PIXEL(smallI,texOffsets[1]);
		sample2 = IMG_NORM_PIXEL(smallI,texOffsets[2]);
	}
	else	{
		sample0 = vec4(1,0,0,1);
		sample1 = vec4(1,0,0,1);
		sample2 = vec4(1,0,0,1);
	}
	vec4 final = vec4((sample0 + sample1 + sample2).rgba / (3.0));
	vec4 original = IMG_NORM_PIXEL(inputImage,texOffsets[0]);
	if (PASSINDEX == 9)	{
		final = mix(original,min((final * original), 1.0), intensity);
		//	First do a gloom,
		//	then a bright / contrast tweak based on the amount
		//	then a bloom
		vec4 tmpColorA = final;
		float bright = intensity * 0.222;
		vec4 tmpColorB = tmpColorA + vec4(bright, bright, bright, bright);
		//	contrast
		float contrast = 1.0 + intensity * (2.893);
		tmpColorA.rgba = ((vec4(2.0) * (tmpColorB.rgba - vec4(0.5))) * vec4(contrast) / vec4(2.0)) + vec4(0.5);
		final = min((tmpColorA + tmpColorA * original), 1.0);
	}
	else	{
		final = mix(sample0, final, intensity);
	}
	gl_FragColor = final;
}
