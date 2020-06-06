/*{
    "CATEGORIES": [
        "Stylize",
        "Feedback",
        "Color Effect"
    ],
    "CREDIT": "by VIDVOX",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": -0.5,
            "LABEL": "Bias",
            "MAX": 0,
            "MIN": -1,
            "NAME": "uBias",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "Scale",
            "MAX": 2,
            "MIN": 0,
            "NAME": "uScale",
            "TYPE": "float"
        },
        {
            "DEFAULT": 5,
            "LABEL": "Ghosts",
            "MAX": 5,
            "MIN": 0,
            "NAME": "uGhosts",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.0125,
            "LABEL": "Ghost Dispersal",
            "MAX": 0.1,
            "MIN": 0,
            "NAME": "uGhostDispersal",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Additive Mode",
            "NAME": "uAdditive",
            "TYPE": "bool"
        },
        {
            "DEFAULT": [
                0.5,
                0.5
            ],
            "LABEL": "Direction",
            "MAX": [
                1,
                1
            ],
            "MIN": [
                0,
                0
            ],
            "NAME": "uDirection",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": [
                0.9,
                0.8,
                0.7,
                1
            ],
            "LABEL": "Lens Color",
            "NAME": "uLensColor",
            "TYPE": "color"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "DESCRIPTION": "Downsample and threshold",
            "HEIGHT": "floor($HEIGHT/1.0)",
            "TARGET": "downsampleAndThresholdImage",
            "WIDTH": "floor($WIDTH/1.0)"
        },
        {
        }
    ]
}
*/




void main()
{

	if (PASSINDEX == 0)	{
		vec2 loc = isf_FragNormCoord;
		gl_FragColor = max(vec4(0.0), IMG_NORM_PIXEL(inputImage,loc) + uBias) * uScale;
	}
	else if (PASSINDEX == 1)	{
		vec2 texcoord = isf_FragNormCoord;
		vec2 texelSize = 1.0 / RENDERSIZE;
		vec2 direction = vec2(1.0) - uDirection;
		vec2 ghostVec = (direction - texcoord) * uGhostDispersal;
		//vec2 direction = vec2(0.5,0.5);
		vec4 result = vec4(0.0);
		for (int i = 0; i < 5; ++i) { 
			if (float(i)>uGhosts)
				break;
			vec2 offset = fract(texcoord + ghostVec * float(i));

			result += IMG_NORM_PIXEL(downsampleAndThresholdImage, offset) * uLensColor;
		}
		//	apply the alpha
		result.rgb = result.rgb * uLensColor.a;
		if (uAdditive)	{
			result = result + IMG_NORM_PIXEL(inputImage, texcoord);
		}
		else	{
			result = result * IMG_NORM_PIXEL(inputImage, texcoord);
		}
		gl_FragColor = result;
	}

}