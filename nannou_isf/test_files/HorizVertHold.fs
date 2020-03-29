/*{
	"DESCRIPTION": "",
	"CREDIT": "by zoidberg",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Geometry Adjustment"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "hHold",
			"LABEL": "Horiz. Hold",
			"TYPE": "float",
			"MIN": -0.45,
			"MAX": 0.45,
			"DEFAULT": 0.0
		},
		{
			"NAME": "vHold",
			"LABEL": "Vert. Hold",
			"TYPE": "float",
			"MIN": -0.45,
			"MAX": 0.45,
			"DEFAULT": 0.0
		},
		{
			"NAME": "flashEvent",
			"TYPE": "event"
		}
	],
	"PASSES": [
		{
			"TARGET":"lastPosition",
			"WIDTH": 1,
			"HEIGHT": 1,
			"FLOAT": true,
			"PERSISTENT": true,
			"DESCRIPTION": "this buffer stores the last frame's x/y offset in the first two components of its only pixel- note that it's requesting a FLOAT target buffer..."
		},
		{
			
		}
	]
	
}*/

void main()
{
	//	if this is the first pass, i'm going to read the position from the "lastPosition" image, and write a new position based on this and the hold variables
	if (PASSINDEX == 0)	{
		vec4		srcPixel = IMG_PIXEL(lastPosition,vec2(0.5));
		//	i'm only using the X and Y components, which are the X and Y offset (normalized) for the frame
		srcPixel.xy = (flashEvent) ? vec2(0.0) : (srcPixel.xy - vec2(hHold,vHold));
		gl_FragColor = mod(srcPixel,1.0);
	}
	//	else this isn't the first pass- read the position value from the buffer which stores it
	else	{
		vec4		lastPosVector = IMG_PIXEL(lastPosition,vec2(0.5));
		vec2		normPixelCoord = mod((isf_FragNormCoord.xy + lastPosVector.xy), 1.0);
		gl_FragColor = IMG_NORM_PIXEL(inputImage,normPixelCoord);
	}
}
