/*{
    "CATEGORIES": [
        "Feedback",
        "Glitch"
    ],
    "CREDIT": "by VIDVOX",
    "DESCRIPTION": "Does a fast faked data-mosh style",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 1,
            "NAME": "update_keyframe",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0.95,
            "MAX": 1,
            "MIN": 0.01,
            "NAME": "update_rate",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 10,
            "MIN": 0,
            "NAME": "sharpen",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 2,
            "MIN": 0,
            "NAME": "blur",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": 0,
            "NAME": "posterize",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABELS": [
                "relative",
                "absolute",
                "difference"
            ],
            "NAME": "mode",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2
            ]
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "HEIGHT": "$HEIGHT/16.0",
            "PERSISTENT": true,
            "TARGET": "keyFrameBuffer1",
            "WIDTH": "$WIDTH/16.0"
        },
        {
            "HEIGHT": "$HEIGHT",
            "PERSISTENT": true,
            "TARGET": "keyFrameBuffer2",
            "WIDTH": "$WIDTH"
        }
    ]
}
*/

#if __VERSION__ <= 120
varying vec2 left_coord;
varying vec2 right_coord;
varying vec2 above_coord;
varying vec2 below_coord;

varying vec2 lefta_coord;
varying vec2 righta_coord;
varying vec2 leftb_coord;
varying vec2 rightb_coord;
#else
in vec2 left_coord;
in vec2 right_coord;
in vec2 above_coord;
in vec2 below_coord;

in vec2 lefta_coord;
in vec2 righta_coord;
in vec2 leftb_coord;
in vec2 rightb_coord;
#endif

void main()
{

	//	first pass: read the "inputImage"- remember, we're drawing to the persistent buffer "keyFrameBuffer1" on the first pass
	//	store it if we're taking in a new keyframe
	if (PASSINDEX == 0)	{
		vec4		stalePixel = IMG_THIS_NORM_PIXEL(keyFrameBuffer1);		
		vec4		freshPixel = IMG_THIS_NORM_PIXEL(inputImage);
		if (update_keyframe==true)	{
			gl_FragColor = freshPixel;
		}
		else	{
			gl_FragColor = stalePixel;
		}
	}
	//	second pass: read from "bufferVariableNameA".  output looks chunky and low-res.
	else if (PASSINDEX == 1)	{
		vec4		stalePixel1 = IMG_THIS_NORM_PIXEL(keyFrameBuffer1);
		vec4		stalePixel2 = IMG_THIS_NORM_PIXEL(keyFrameBuffer2);
		vec4 color = IMG_THIS_NORM_PIXEL(inputImage);
		vec4 colorL = IMG_NORM_PIXEL(inputImage, left_coord);
		vec4 colorR = IMG_NORM_PIXEL(inputImage, right_coord);
		vec4 colorA = IMG_NORM_PIXEL(inputImage, above_coord);
		vec4 colorB = IMG_NORM_PIXEL(inputImage, below_coord);

		vec4 colorLA = IMG_NORM_PIXEL(inputImage, lefta_coord);
		vec4 colorRA = IMG_NORM_PIXEL(inputImage, righta_coord);
		vec4 colorLB = IMG_NORM_PIXEL(inputImage, leftb_coord);
		vec4 colorRB = IMG_NORM_PIXEL(inputImage, rightb_coord);

		vec4 final = color;
		vec4 diff = (color - stalePixel2);
		if (blur > 0.0)	{
			float blurAmount = blur / 8.0;
			diff = diff * (1.0 - blur) + (colorL + colorR + colorA + colorB + colorLA + colorRA + colorLB + colorRB) * blurAmount;
		}
		
		if (mode == 0)	{
			final = mix(stalePixel2,(diff + stalePixel1),update_rate);
		}
		else if (mode == 1)	{
			final = mix(stalePixel2,abs(diff) + stalePixel1,update_rate);
		}
		else if (mode == 2)	{
			final = mix(stalePixel2,abs(diff + stalePixel2) * stalePixel1,update_rate);
		}
		
		if (posterize > 0.0)	{
			float qLevel = 128.0 - posterize * 126.0;
			final = floor(final*qLevel)/qLevel;
		}
		
		if (sharpen > 0.0)	{
			final = final + sharpen * (8.0*color - colorL - colorR - colorA - colorB - colorLA - colorRA - colorLB - colorRB);
		}
		
		gl_FragColor = final;
	}
}
