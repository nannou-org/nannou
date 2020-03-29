/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Sharpen"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "intensity",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 2.0,
			"DEFAULT": 1.0
		}
	]
}*/

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

float gray(vec4 n)
{
	return (n.r + n.g + n.b)/3.0;
}

void main()
{

	vec4 color = IMG_THIS_PIXEL(inputImage);
	float colorL = gray(IMG_NORM_PIXEL(inputImage, left_coord));
	float colorR = gray(IMG_NORM_PIXEL(inputImage, right_coord));
	float colorA = gray(IMG_NORM_PIXEL(inputImage, above_coord));
	float colorB = gray(IMG_NORM_PIXEL(inputImage, below_coord));

	float colorLA = gray(IMG_NORM_PIXEL(inputImage, lefta_coord));
	float colorRA = gray(IMG_NORM_PIXEL(inputImage, righta_coord));
	float colorLB = gray(IMG_NORM_PIXEL(inputImage, leftb_coord));
	float colorRB = gray(IMG_NORM_PIXEL(inputImage, rightb_coord));

	vec4 final = color + color * intensity * (8.0*gray(color) - colorL - colorR - colorA - colorB - colorLA - colorRA - colorLB - colorRB);
	
	gl_FragColor = final;
}