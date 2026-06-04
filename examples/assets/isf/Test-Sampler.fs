/*{
	"DESCRIPTION": "demonstrates the use of multiple image-type inputs",
	"CREDIT": "by zoidberg",
	"ISFVSN": "2.0",
	"CATEGORIES": [
		"TEST-GLSL FX"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "blendImage",
			"TYPE": "image"
		}
	]
}*/

void main()
{
	//	these two variable declarations are functionally identical (the only difference is that THIS_PIXEL is a non-dependent texture lookup)
	//vec4		srcPixel = IMG_PIXEL(inputImage, gl_FragCoord.xy);	//	returns a vec4 with the colors of the pixel in sampler "inputImage" at (non-normalized) texture coordinates "srcCoord"
	//vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	
	//	these two variable declarations are also identical.  though they behave similarly to the above declarations in this specific filter, NORM and non-NORM pixel lookups are fundamentally different and behave differently under other circumstances.
	//vec4		srcPixel = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord);
	vec4		srcPixel = IMG_THIS_NORM_PIXEL(inputImage);
	
	vec4		blendPixel = IMG_THIS_NORM_PIXEL(blendImage);	//	returns a vec4 with the colors of the pixel in sampler "inputImage" at (normalized) texture coordinates "blendCoord".
	gl_FragColor = srcPixel/2.0 + blendPixel/2.0;
}
