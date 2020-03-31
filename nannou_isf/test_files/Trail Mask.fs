/*{
    "CATEGORIES": [
        "Glitch"
    ],
    "CREDIT": "by VIDVOX",
    "DESCRIPTION": "Pixels update only based on the masking image",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "LABEL": "mask image",
            "NAME": "maskImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "LABEL": "mask size mode",
            "LABELS": [
                "Fit",
                "Fill",
                "Stretch",
                "Copy"
            ],
            "NAME": "maskSizingMode",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2,
                3
            ]
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": -1,
            "NAME": "bright",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 4,
            "MIN": -4,
            "NAME": "contrast",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "NAME": "RGB_mode",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "PERSISTENT": true,
            "TARGET": "bufferVariableNameA"
        }
    ]
}
*/



//const vec4		lumcoeff = vec4(0.299, 0.587, 0.114, 0.0);
const vec4 		lumcoeff = vec4(0.2126, 0.7152, 0.0722, 0.0);

//	'a' and 'b' are rects (x and y are the originx, z and w are the width and height)
//	'm' is the sizing mode as described above (fit/fill/stretch/copy)
vec4 RectThatFitsRectInRect(vec4 a, vec4 b, int m);


void main()
{
	vec4	freshPixel = IMG_PIXEL(inputImage,gl_FragCoord.xy);
	//	If we're on the line, update, otherwise use the stale pixel
	vec4 	stalePixel = IMG_PIXEL(bufferVariableNameA,gl_FragCoord.xy);

	
	//	get the rect of the mask image after it's been resized according to the passed sizing mode.  this is in pixel coords relative to the rendering space!
	vec4		rectOfResizedMaskImage = RectThatFitsRectInRect(vec4(0.0, 0.0, _maskImage_imgRect.z, _maskImage_imgRect.w), vec4(0,0,RENDERSIZE.x,RENDERSIZE.y), maskSizingMode);
	//vec4		rectOfResizedMaskImage = RectThatFitsRectInRect(vec4(0.0, 0.0, IMG_SIZE(maskImage).x, IMG_SIZE(maskImage).y), vec4(0,0,RENDERSIZE.x,RENDERSIZE.y), maskSizingMode);
	//	i know the pixel coords of this frag in the render space- convert this to NORMALIZED texture coords for the resized mask image
	vec2		normMaskSrcCoord;
	normMaskSrcCoord.x = (gl_FragCoord.x-rectOfResizedMaskImage.x)/rectOfResizedMaskImage.z;
	normMaskSrcCoord.y = (gl_FragCoord.y-rectOfResizedMaskImage.y)/rectOfResizedMaskImage.w;
	
	//	get the color of the pixel from the mask image for these normalized coords
	vec4		tmpColorA = IMG_NORM_PIXEL(maskImage, normMaskSrcCoord);
	
	//	apply bright/contrast to this pixel value
	vec4		tmpColorB = tmpColorA + vec4(bright, bright, bright, 0.0);
	tmpColorA.rgb = ((vec3(2.0) * (tmpColorB.rgb - vec3(0.5))) * vec3(contrast) / vec3(2.0)) + vec3(0.5);
	tmpColorA.a = ((2.0 * (tmpColorB.a - 0.5)) * abs(contrast) / 2.0) + 0.5;
	
	
	vec4		result;
	if (RGB_mode)	{
		result.a = freshPixel.a;
		result.rgb = mix(freshPixel.rgb, stalePixel.rgb, clamp(tmpColorA.rgb,0.0,1.0));
	}
	//	get the luminance of this pixel value: this will be the new alpha for the source pixel
	//	use the luminance to mix between the fresh pixel and the stale one
	else	{
		float		luminance = dot(tmpColorA,lumcoeff);
		result = mix(freshPixel, stalePixel, clamp(luminance,0.0,1.0));
	}

	gl_FragColor = result;
}


//	rect that fits 'a' in 'b' using sizing mode 'm'
vec4 RectThatFitsRectInRect(vec4 a, vec4 b, int m)	{
	float		bAspect = b.z/b.w;
	float		aAspect = a.z/a.w;
	if (aAspect==bAspect)	{
		return b;
	}
	vec4		returnMe = vec4(0.0);
	//	fit
	if (m==0)	{
		//	if the rect i'm trying to fit stuff *into* is wider than the rect i'm resizing
		if (bAspect > aAspect)	{
			returnMe.w = b.w;
			returnMe.z = returnMe.w * aAspect;
		}
		//	else if the rect i'm resizing is wider than the rect it's going into
		else if (bAspect < aAspect)	{
			returnMe.z = b.z;
			returnMe.w = returnMe.z / aAspect;
		}
		else	{
			returnMe.z = b.z;
			returnMe.w = b.w;
		}
		returnMe.x = (b.z-returnMe.z)/2.0+b.x;
		returnMe.y = (b.w-returnMe.w)/2.0+b.y;
	}
	//	fill
	else if (m==1)	{
		//	if the rect i'm trying to fit stuff *into* is wider than the rect i'm resizing
		if (bAspect > aAspect)	{
			returnMe.z = b.z;
			returnMe.w = returnMe.z / aAspect;
		}
		//	else if the rect i'm resizing is wider than the rect it's going into
		else if (bAspect < aAspect)	{
			returnMe.w = b.w;
			returnMe.z = returnMe.w * aAspect;
		}
		else	{
			returnMe.z = b.z;
			returnMe.w = b.w;
		}
		returnMe.x = (b.z-returnMe.z)/2.0+b.x;
		returnMe.y = (b.w-returnMe.w)/2.0+b.y;
	}
	//	stretch
	else if (m==2)	{
		returnMe = vec4(b.x, b.y, b.z, b.w);
	}
	//	copy
	else if (m==3)	{
		returnMe.z = float(int(a.z));
		returnMe.w = float(int(a.w));
		returnMe.x = float(int((b.z-returnMe.z)/2.0+b.x));
		returnMe.y = float(int((b.w-returnMe.w)/2.0+b.y));
	}
	return returnMe;
}
