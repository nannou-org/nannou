/*{
    "CATEGORIES": [
        "Drawing"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "Draw images one pixel at a time",
    "INPUTS": [
        {
            "NAME": "moveUp",
            "TYPE": "event"
        },
        {
            "NAME": "moveDown",
            "TYPE": "event"
        },
        {
            "NAME": "moveLeft",
            "TYPE": "event"
        },
        {
            "NAME": "moveRight",
            "TYPE": "event"
        },
        {
            "DEFAULT": [
                0.5,
                0.5,
                0.5,
                1
            ],
            "NAME": "penColor",
            "TYPE": "color"
        },
        {
            "DEFAULT": 0.05,
            "MAX": 1,
            "MIN": 0,
            "NAME": "penSize",
            "TYPE": "float"
        },
        {
            "NAME": "resetPosition",
            "TYPE": "event"
        },
        {
            "NAME": "clearBuffer",
            "TYPE": "event"
        },
        {
            "DEFAULT": [
                0,
                0,
                0,
                0
            ],
            "NAME": "clearColor",
            "TYPE": "color"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "FLOAT": true,
            "HEIGHT": "1",
            "PERSISTENT": true,
            "TARGET": "currentPosition",
            "WIDTH": "1"
        },
        {
            "PERSISTENT": true,
            "TARGET": "lastState"
        }
    ]
}
*/


float round (float x)	{
	if (fract(x) < 0.5)
		return floor(x);
	else
		return ceil(x);	
}


void main()	{
	vec4		inputPixelColor = vec4(0.0);
	
	if (PASSINDEX == 0)	{
		if ((FRAMEINDEX==0)||(resetPosition))
			inputPixelColor = vec4(0.5,0.5,0.0,1.0);
		else
			inputPixelColor = IMG_THIS_PIXEL(currentPosition);
		
		vec2		pos = inputPixelColor.rg;
		vec2		outputSize = IMG_SIZE(lastState);
		vec2		penSizeInPixels = vec2(penSize * min(outputSize.x,outputSize.y));
		penSizeInPixels.x = (penSizeInPixels.x < 1.0) ? (1.0) : (penSizeInPixels.x);
		penSizeInPixels.y = (penSizeInPixels.y < 1.0) ? (1.0) : (penSizeInPixels.y);
		vec2		normalizedPenSize = penSizeInPixels / outputSize;
		
		if ((moveUp)&&(pos.y<1.0-normalizedPenSize.y))	{
			pos.y = pos.y + normalizedPenSize.y;
		}
		else if ((moveDown)&&(pos.y>0.0))	{
			pos.y = pos.y - normalizedPenSize.y;
		}
		else if ((moveLeft)&&(pos.x>0.0))	{
			pos.x = pos.x - normalizedPenSize.x;
		}
		else if ((moveRight)&&(pos.x<1.0-normalizedPenSize.x))	{
			pos.x = pos.x + normalizedPenSize.x;
		}
		pos = min(pos,vec2(1.0)-normalizedPenSize);
		pos = max(pos,vec2(0.0));
		inputPixelColor.rg = pos;
	}
	else if (PASSINDEX == 1)	{
		if ((FRAMEINDEX==0)||(clearBuffer))	{
			inputPixelColor = clearColor;
		}
		else	{
			vec4		posPixel = IMG_PIXEL(currentPosition,vec2(0.0));
			vec2		pos = posPixel.rg * RENDERSIZE;
			vec2		penSizeInPixels = vec2(penSize * min(RENDERSIZE.x,RENDERSIZE.y));
			penSizeInPixels.x = (penSizeInPixels.x < 1.0) ? (1.0) : (penSizeInPixels.x);
			penSizeInPixels.y = (penSizeInPixels.y < 1.0) ? (1.0) : (penSizeInPixels.y);
			//pos.x = round(pos.x);
			//pos.y = round(pos.y);
			if (((gl_FragCoord.x) >= pos.x)&&((gl_FragCoord.y) >= pos.y)&&((gl_FragCoord.x) < pos.x + penSizeInPixels.x)&&((gl_FragCoord.y) < pos.y + penSizeInPixels.y))
				inputPixelColor = penColor;
			else
				inputPixelColor = IMG_THIS_PIXEL(lastState);
		}
	}
	
	gl_FragColor = inputPixelColor;
}
