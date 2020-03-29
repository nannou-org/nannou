/*{
    "CATEGORIES": [
        "Feedback",
        "Glitch"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "This does a feedback motion blur based on the brightness of pixels to create an analog recording comet trails effect",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "absorptionRate",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.02,
            "MAX": 1,
            "MIN": 0,
            "NAME": "dischargeRate",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "FLOAT": true,
            "PERSISTENT": true,
            "TARGET": "feedbackBuffer"
        }
    ]
}
*/



//	Comet Tails is a specific type of Image Lag
//	From https://bavc.github.io/avaa/artifacts/image_lag.html
/*
Image lag occurs in video recorded or displayed using certain types of pick-up devices and cameras, including the Vidicon picture tube, among others.
This type of camera tube captures light radiating from a scene through a lens and projects it onto a photoconductive target, creating a charge-density pattern which is scanned using low-velocity electrons.
The resulting image can be amplified and recorded to tape or output to a video monitor.
The electrical charge remains present on the target until it is re-scanned or the charge dissipates.
*/



const vec4  kRGBToYPrime = vec4 (0.299, 0.587, 0.114, 0.0);
const vec4  kRGBToI     = vec4 (0.596, -0.275, -0.321, 0.0);
const vec4  kRGBToQ     = vec4 (0.212, -0.523, 0.311, 0.0);

const vec4  kYIQToR   = vec4 (1.0, 0.956, 0.621, 0.0);
const vec4  kYIQToG   = vec4 (1.0, -0.272, -0.647, 0.0);
const vec4  kYIQToB   = vec4 (1.0, -1.107, 1.704, 0.0);


vec3 rgb2yiq(vec3 c)	{
    float   YPrime = dot (c, kRGBToYPrime.rgb);
    float   I = dot (c, kRGBToI.rgb);
    float   Q = dot (c, kRGBToQ.rgb);
    return vec3(YPrime,I,Q);
}

vec3 yiq2rgb(vec3 c)	{
    vec3    yIQ   = vec3 (c.r, c.g, c.b);
    return vec3(dot (yIQ, kYIQToR.rgb),dot (yIQ, kYIQToG.rgb),dot (yIQ, kYIQToB.rgb));
}




void main()
{
	vec4		freshPixel = IMG_PIXEL(inputImage,gl_FragCoord.xy);
	vec4		stalePixel = IMG_PIXEL(feedbackBuffer,gl_FragCoord.xy);
	//	the input gamma range is 0.0-1.0 (normalized).  the actual gamma range i want to use is 0.0 - 5.0.
	//	however, actual gamma 0.0-1.0 is just as interesting as actual gamma 1.0-5.0, so we scale the normalized input to match...
	float		realGamma = (absorptionRate<=0.5) ? (absorptionRate * 2.0) : (((absorptionRate-0.5) * 2.0 * 4.0) + 1.0);
	vec4		tmpColorA = stalePixel;
	vec4		tmpColorB;
	tmpColorB.rgb = pow(tmpColorA.rgb, vec3(1.0/realGamma));
	tmpColorB.a = tmpColorA.a;
	
	float		feedbackLevel = rgb2yiq(tmpColorB.rgb).r;
	//	if the new pixel is brighter, it immediately fills up the buffer to its level
	if (rgb2yiq(freshPixel.rgb).r > feedbackLevel)	{
		feedbackLevel = 0.0;
	}
	//	otherwise, discharge
	else	{
		feedbackLevel *= (1.0 - dischargeRate);
	}
	
	gl_FragColor = mix(freshPixel,stalePixel,feedbackLevel);
}
