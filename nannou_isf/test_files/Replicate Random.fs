/*{
    "CATEGORIES": [
        "Tile Effect"
    ],
    "CREDIT": "",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "NAME": "randomSeed",
            "TYPE": "float"
        },
        {
            "DEFAULT": 5,
            "MAX": 15,
            "MIN": 1,
            "NAME": "repetitions",
            "TYPE": "float"
        },
        {
            "NAME": "randomizeOpacity",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2"
}
*/


float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}


vec2 paddedZoomedPosition(vec2 loc, float zl, vec2 c, float p)	{
	vec2		returnMe = loc;
	float		zoomMult = (1.0/zl);
	vec2		modifiedCenter = 2.0*(1.0+p)*c/RENDERSIZE-(1.0+p);
	float		modifiedPadding = p;
	
	returnMe.x = (returnMe.x)*zoomMult + p/2.0 - modifiedCenter.x;
	returnMe.y = (returnMe.y)*zoomMult + p/2.0 - modifiedCenter.y;
	returnMe.x = mod(returnMe.x,1.0+modifiedPadding) - p/2.0;
	returnMe.y = mod(returnMe.y,1.0+modifiedPadding) - p/2.0;
	
	return returnMe;
}

vec2 randomPaddedZoomedPositionWithSeed(vec2 loc, vec2 seed)	{
	float		minZoomLevel = (4.0/RENDERSIZE.x);
	vec2		zSeed = seed * vec2(0.128,9.21) + vec2(1.42,2.17);
	vec2		cSeed1 = seed * vec2(0.436,0.931) + vec2(2.76,3.779);
	vec2		cSeed2 = seed * vec2(2.831,2.173) + vec2(1.73,6.256);
	float		modZoom = 0.9*rand(zSeed);
	vec2		modCenter = RENDERSIZE*vec2(rand(cSeed1),rand(cSeed2));
	float		modPad = 0.5*rand(seed);
	modZoom = (modZoom < minZoomLevel) ? minZoomLevel : modZoom;
	
	vec2		returnMe = paddedZoomedPosition(loc,modZoom,modCenter,modPad);
	
	return returnMe;
}




void main()	{
	vec4		inputPixelColor = vec4(0.0);

	int			depth = int(repetitions);
	vec2		loc = isf_FragNormCoord;
	
	for (int i = 0;i < 15;++i)	{
		if (i >= depth)
			break;
		vec2		tmpSeed = vec2((1.12+float(i))*randomSeed+1.37,(1.92+float(i))*randomSeed+1.37);
		float		modOpacity = (randomizeOpacity) ? 0.25+0.75*rand(tmpSeed) : 1.0;
		loc = randomPaddedZoomedPositionWithSeed(isf_FragNormCoord,tmpSeed);
		if ((loc.x < 0.0)||(loc.y < 0.0)||(loc.x > 1.0)||(loc.y > 1.0))	{
			//inputPixelColor = vec4(0.0);
		}
		else	{
			vec4	tmpColor = IMG_NORM_PIXEL(inputImage,loc);
			inputPixelColor.rgb = inputPixelColor.rgb + tmpColor.rgb * tmpColor.a * modOpacity;
			inputPixelColor.a += (tmpColor.a * modOpacity);
			if (inputPixelColor.a > 0.99)	{
				break;
			}
		}
		
	}
	
	gl_FragColor = inputPixelColor;
}
