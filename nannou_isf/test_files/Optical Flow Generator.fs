/*{
    "CATEGORIES": [
        "Masking",
        "Utility"
    ],
    "CREDIT": "by VIDVOX / v002 / Andrew Benson",
    "DESCRIPTION": "Creates a raw optical flow mask from the input image",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 10,
            "LABEL": "Scale",
            "MAX": 50,
            "MIN": 0,
            "NAME": "inputScale",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.01,
            "LABEL": "Offset",
            "MAX": 0.5,
            "MIN": 0,
            "NAME": "inputOffset",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.2,
            "LABEL": "Noise Removal",
            "MAX": 1,
            "MIN": 0,
            "NAME": "inputLambda",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "maskOpacity",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "TARGET": "maskBuffer"
        },
        {
            "PERSISTENT": true,
            "TARGET": "delayBuffer"
        },
        {
        }
    ]
}
*/


//	based on v002 Optical Flow which is itself a port of Andrew Bensons HS Flow implementation on the GPU.
//	https://github.com/v002/v002-Optical-Flow

const vec4 coeffs = vec4(0.2126, 0.7152, 0.0722, 1.0);

float gray(vec4 n)
{
	return (n.r + n.g + n.b)/3.0;
}

void main()
{
	//	on the first pass generate the mask using the previous delayBuffer and inputImage
	//	on the 2nd pass update the delayBuffer to hold inputImage
	//	on the 3rd pass output the new mask
	if (PASSINDEX == 0)	{
		//	convert to grayscale
		vec4 a = IMG_THIS_PIXEL(inputImage) * coeffs;
		float brightness = gray(a);
		a = vec4(brightness);
		vec4 b = IMG_THIS_PIXEL(delayBuffer) * coeffs;
		brightness = gray(b);
		b = vec4(brightness);
		
		vec2 x1 = vec2(inputOffset * RENDERSIZE.x, 0.0);
		vec2 y1 = vec2(0.0,inputOffset * RENDERSIZE.y / 2.0);
		vec2 texcoord0 = isf_FragNormCoord.xy * RENDERSIZE;
		vec2 texcoord1 = isf_FragNormCoord.xy * RENDERSIZE;
		
		//get the difference
		vec4 curdif = b-a;
	
		//calculate the gradient
		vec4 gradx = IMG_PIXEL(delayBuffer, texcoord1+x1)-IMG_PIXEL(delayBuffer, texcoord1-x1);
		gradx += IMG_PIXEL(inputImage, texcoord0+x1)-IMG_PIXEL(inputImage, texcoord0-x1);
	
		vec4 grady = IMG_PIXEL(delayBuffer, texcoord1+y1)-IMG_PIXEL(delayBuffer, texcoord1-y1);
		grady += IMG_PIXEL(inputImage, texcoord0+y1)-IMG_PIXEL(inputImage, texcoord0-y1);
	
		vec4 gradmag = sqrt((gradx*gradx)+(grady*grady)+vec4(inputLambda));

		vec4 vx = curdif*(gradx/gradmag);
		float vxd = gray(vx);//assumes greyscale
		//format output for flowrepos, out(-x,+x,-y,+y)
		vec2 xout = vec2(max(vxd,0.),abs(min(vxd,0.)))*inputScale;

		vec4 vy = curdif*(grady/gradmag);
		float vyd = gray(vy);//assumes greyscale
		//format output for flowrepos, out(-x,+x,-y,+y)
		vec2 yout = vec2(max(vyd,0.),abs(min(vyd,0.)))*inputScale;
	
		vec4 mask = clamp(vec4(xout.xy,yout.xy), 0.0, 1.0);
		gl_FragColor = mask;
	}
	else if (PASSINDEX == 1)	{
		gl_FragColor = IMG_THIS_PIXEL(inputImage);
	}
	else	{
		//	NOW DO SOMETHING WITH THE MASK
		vec4 mask = IMG_THIS_NORM_PIXEL(maskBuffer);
		//mask.a = 1.0;
		mask.a = mix(mask.a, 1.0, maskOpacity);
		gl_FragColor = mask;
	}
}
