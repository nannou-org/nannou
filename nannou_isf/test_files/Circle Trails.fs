/*{
    "CATEGORIES": [
        "Drawing"
    ],
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "DEFAULT": [
                0,
                0
            ],
            "MAX": [
                1,
                1
            ],
            "MIN": [
                0,
                0
            ],
            "NAME": "pointInput",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": [
                0.9596965909004211,
                0.5817266273340564,
                0.17662368847084817,
                1
            ],
            "NAME": "startColor",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0.9623754620552063,
                0.9023552320837249,
                0.28796788144265206,
                1
            ],
            "NAME": "endColor",
            "TYPE": "color"
        },
        {
            "DEFAULT": 0.1,
            "MAX": 0.5,
            "MIN": 0,
            "NAME": "startSize",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 0.5,
            "MIN": 0,
            "NAME": "endSize",
            "TYPE": "float"
        },
        {
            "DEFAULT": 16,
            "MAX": 32,
            "MIN": 1,
            "NAME": "trailCount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 0.1,
            "MIN": 0,
            "NAME": "strokeSize",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0,
                0,
                0,
                1
            ],
            "NAME": "strokeColor",
            "TYPE": "color"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "HEIGHT": "1",
            "PERSISTENT": true,
            "TARGET": "pointsBuffer",
            "WIDTH": "$trailCount"
        },
        {
            "DESCRIPTION": "this empty pass is rendered at the same rez as whatever you are running the ISF filter at- the previous step rendered an image at one-sixteenth the res, so this step ensures that the output is full-size"
        }
    ],
    "VSN": ""
}
*/

float drawCircle(vec2 pt, vec2 circlePosition, float circleSize){
    if (distance(pt, circlePosition)<circleSize){
        return distance(pt, circlePosition) / circleSize;
    }
    return 0.0;
}

void main()	{
	vec4		returnMe = vec4(0.0);
	
	if (PASSINDEX == 0)	{
		vec2	loc = isf_FragNormCoord.xy;
		float	val = 1.0 / floor(trailCount);
		if (loc.x < val)	{
			returnMe = vec4(pointInput.x,pointInput.y,1.0,1.0);
		}
		else	{
			loc.x = loc.x - val;
			returnMe = IMG_NORM_PIXEL(pointsBuffer,loc);
		}
	}
	else if (PASSINDEX == 1)	{
		float	inCircle = 0.0;
		float	aspectRatio = RENDERSIZE.x / RENDERSIZE.y;
		vec2	loc = isf_FragNormCoord * aspectRatio;
		float	trailFloor = floor(trailCount);
		int 	j = 0;
		float	val = 1.0 / (trailFloor-1.0);
		
		for (int i = 0;i < 32;++i)	{
			
			float	cSize = mix(startSize,endSize,float(i)/trailFloor);
			vec2	pos = vec2(float(i)*val,0.5);
			vec4	ptInfo = IMG_NORM_PIXEL(pointsBuffer,pos);
			if (ptInfo.g > 0.0)
				inCircle += drawCircle(loc,ptInfo.xy*aspectRatio,cSize);
			
			if (inCircle > 0.0)	{
				//inCircle = 1.0;
				j = i;
				break;
			}
			
			if (i == int(trailCount))	{
				j = i;
				break;
			}
			
		}
		
		if (inCircle > 0.0)	{
			returnMe = mix(startColor,endColor,float(j)/floor(trailCount));
			
			if ((strokeSize > 0.0)&&(inCircle > 1.0 - strokeSize))	{
				returnMe = strokeColor;
			}
			
		}
		
	}

	gl_FragColor = returnMe;
}
