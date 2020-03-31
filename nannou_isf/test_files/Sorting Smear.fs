/*{
    "CATEGORIES": [
        "Glitch",
        "Feedback"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "LABEL": "Flush Buffer",
            "NAME": "resetInput",
            "TYPE": "event"
        },
        {
            "DEFAULT": 0.25,
            "LABEL": "Adapt Rate",
            "MAX": 1,
            "MIN": 0,
            "NAME": "adaptLevel",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Sort Rate",
            "MAX": 1,
            "MIN": 0,
            "NAME": "sortRate",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "H Sort",
            "NAME": "horizontalSort",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 1,
            "LABEL": "V Sort",
            "NAME": "verticalSort",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "PERSISTENT": true,
            "TARGET": "lastRender"
        }
    ]
}
*/

#if __VERSION__ <= 120
varying vec2 left_coord;
varying vec2 right_coord;
varying vec2 above_coord;
varying vec2 below_coord;
#else
in vec2 left_coord;
in vec2 right_coord;
in vec2 above_coord;
in vec2 below_coord;
#endif



void main()
{
	vec4 result = vec4(0.0);
	vec4 color = IMG_THIS_NORM_PIXEL(inputImage);
	//float localAdaptRate = 1.0 - pow(adaptLevel,2.0);
	float localAdaptRate = adaptLevel;
	//	if the frame index is 0, or the reset event was triggered, use the original color
	if ((FRAMEINDEX <= 1)||(resetInput))	{
		result = color;
	}
	else	{
		vec4 oldColor = IMG_THIS_NORM_PIXEL(lastRender);
		color = mix(color, oldColor, localAdaptRate);
		result = color;
		float	b0 = (color.r + color.b + color.g) / 3.0;

		if (verticalSort)	{
			vec4 colorA = IMG_NORM_PIXEL(lastRender, above_coord);
			vec4 colorB = IMG_NORM_PIXEL(lastRender, below_coord);
			float	bA = (colorA.r + colorA.b + colorA.g) / 3.0;
			float	bB = (colorB.r + colorB.b + colorB.g) / 3.0;
			float	localSortRate = sortRate;
			//	if this pixel is brighter to the one on the left, we are swapping with it
			if (b0 < bA)	{
				result = mix(result, colorA, localSortRate);
			}
			//	if this pixel is not as bright as the one to the right, we are swapping with it
			if (b0 > bB)	{
				result = mix(result, colorB, localSortRate);
			}
		}
		if (horizontalSort)	{
			vec4	colorL = IMG_NORM_PIXEL(lastRender, left_coord);
			vec4	colorR = IMG_NORM_PIXEL(lastRender, right_coord);
			float	bL = (colorL.r + colorL.b + colorL.g) / 3.0;
			float	bR = (colorR.r + colorR.b + colorR.g) / 3.0;
			float	localSortRate = (verticalSort) ? 0.5 : 1.0;
			localSortRate = localSortRate * sortRate;
			//	if this pixel is brighter to the one on the left, we are swapping with it
			if (b0 > bL)	{
				result = mix(result, colorL, localSortRate);
			}
			//	if this pixel is not as bright as the one to the right, we are swapping with it
			if (b0 < bR)	{
				result = mix(result, colorR, localSortRate);
			}
		}
	}	
	gl_FragColor = result;
}