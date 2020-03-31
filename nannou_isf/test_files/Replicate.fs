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
            "DEFAULT": 0.5,
            "IDENTITY": 1,
            "MAX": 2,
            "MIN": 0,
            "NAME": "startSize",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "startOpacity",
            "TYPE": "float"
        },
        {
            "NAME": "startCenter",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": 0,
            "NAME": "startPadding",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 2,
            "MIN": 0,
            "NAME": "endSize",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "endOpacity",
            "TYPE": "float"
        },
        {
            "NAME": "endCenter",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": 0.1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "endPadding",
            "TYPE": "float"
        },
        {
            "DEFAULT": 5,
            "MAX": 15,
            "MIN": 1,
            "NAME": "repetitions",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/


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


void main()	{
	vec4		inputPixelColor = vec4(0.0);

	int			depth = int(repetitions);
	vec2		loc = isf_FragNormCoord;
	float		minZoomLevel = (1.0/RENDERSIZE.x);
	float		startZoomLevel = (startSize < minZoomLevel) ?  minZoomLevel : startSize;
	float		endZoomLevel = (endSize < minZoomLevel) ?  minZoomLevel : endSize;
	float		zoomIncrement = (depth < 2) ? 0.0 : (endZoomLevel - startZoomLevel)/float(depth-1);
	vec2		centerIncrement = (depth < 2) ? vec2(0.0) : (endCenter - startCenter)/float(depth-1);
	float		paddingIncrement = (depth < 2) ? 0.0 : (endPadding - startPadding)/float(depth-1);
	float		opacityIncrement = (depth < 2) ? 0.0 : (endOpacity - startOpacity)/float(depth-1);
	
	for (int i = 0;i < 15;++i)	{
		if (i >= depth)
			break;
		float	modZoom = startZoomLevel + zoomIncrement * float(i);
		vec2	modCenter = startCenter + centerIncrement * float(i);
		float	modPad = startPadding + paddingIncrement * float(i);
		float	modOpacity = startOpacity + opacityIncrement * float(i);
		modOpacity = clamp(modOpacity,0.0,1.0);
		loc = paddedZoomedPosition(isf_FragNormCoord,modZoom,modCenter,modPad);
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
