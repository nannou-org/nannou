/*{
    "CATEGORIES": [
        "Noise"
    ],
    "CREDIT": "by VIDVOX",
    "DESCRIPTION": "Pixels that change become noise until they match the input again",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "adaptRate",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.05,
            "MAX": 1,
            "MIN": 0,
            "NAME": "threshold",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "NAME": "useRGBA",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "PERSISTENT": true,
            "TARGET": "adaptiveBuffer1"
        },
        {
        }
    ]
}
*/


float	minThresh = 3.0/255.0;


float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}
float steppedRand(vec2 co){
    return floor(256.0*rand(co))/255.0;
}
vec4 randColor(vec2 co){
	vec4	c = vec4(0.0);
	c.r = steppedRand(co+0.234);
	c.g = steppedRand(co+0.193);
	c.b = steppedRand(co+0.625);
	c.a = 1.0;
    return c;
}

float luma(vec4 c)	{
	return c.a*(c.r+c.g+c.b)/3.0;	
}


void main()
{
	
	vec4		freshPixel = IMG_PIXEL(inputImage,gl_FragCoord.xy);
	vec4		stalePixel = IMG_PIXEL(adaptiveBuffer1,gl_FragCoord.xy);
	if (useRGBA == false)	{
		float		freshLuma = luma(freshPixel);
		float		staleLuma = luma(stalePixel);
		float		thresh = (threshold < minThresh) ? minThresh : threshold;
		if (abs(freshLuma-staleLuma)>=thresh)	{
			stalePixel = randColor(gl_FragCoord.xy+TIME);
		}
	}
	else	{
		float	thresh = (threshold < minThresh) ? minThresh : threshold;
		if (abs(freshPixel.r-stalePixel.r)>=thresh)	{
			stalePixel.r = steppedRand(gl_FragCoord.xy+TIME+1.234);
		}
		if (abs(freshPixel.g-stalePixel.g)>=thresh)	{
			stalePixel.g = steppedRand(gl_FragCoord.xy+TIME+2.193);
		}
		if (abs(freshPixel.b-stalePixel.b)>=thresh)	{
			stalePixel.b = steppedRand(gl_FragCoord.xy+TIME+3.625);
		}
		if (abs(freshPixel.a-stalePixel.a)>=thresh)	{
			stalePixel.a = steppedRand(gl_FragCoord.xy+TIME+4.479);
		}
	}
	gl_FragColor = mix(stalePixel,freshPixel,adaptRate);

}
