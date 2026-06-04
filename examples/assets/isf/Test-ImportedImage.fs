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
		}
	],
	"IMPORTED": {
		"blendImage": {
			"PATH": "Hexagon.tiff"
		}
	}
}*/

void main()
{
	vec4		srcPixel = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord);
	vec4		blendPixel = IMG_NORM_PIXEL(blendImage, isf_FragNormCoord);
	
	gl_FragColor = (srcPixel + blendPixel)/2.0;
}
