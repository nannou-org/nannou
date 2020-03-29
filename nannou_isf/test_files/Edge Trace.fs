/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Stylize"
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
			"MAX": 50.0,
			"DEFAULT": 2.5
		},
		{
			"NAME": "spread",
			"TYPE": "float",
			"MIN": 1.0,
			"MAX": 20.0,
			"DEFAULT": 5.0
		},
		{
			"NAME": "invert_lines",
			"TYPE": "bool",
			"DEFAULT": 0
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
	vec4 colorL = IMG_NORM_PIXEL(inputImage, left_coord);
	vec4 colorR = IMG_NORM_PIXEL(inputImage, right_coord);
	vec4 colorA = IMG_NORM_PIXEL(inputImage, above_coord);
	vec4 colorB = IMG_NORM_PIXEL(inputImage, below_coord);

	vec4 colorLA = IMG_NORM_PIXEL(inputImage, lefta_coord);
	vec4 colorRA = IMG_NORM_PIXEL(inputImage, righta_coord);
	vec4 colorLB = IMG_NORM_PIXEL(inputImage, leftb_coord);
	vec4 colorRB = IMG_NORM_PIXEL(inputImage, rightb_coord);

	float gx = (-1.0 * gray(colorLA)) + (-1.0 * gray(colorL)) + (-1.0 * gray(colorLB)) + (1.0 * gray(colorRA)) + (1.0 * gray(colorR)) + (1.0 * gray(colorRB));
	float gy = (1.0 * gray(colorLA)) + (1.0 * gray(colorA)) + (1.0 * gray(colorRA)) + (-1.0 * gray(colorRB)) + (-1.0 * gray(colorB)) + (-1.0 * gray(colorLB));
	
	vec4	blurred = (colorL + colorR + colorA + colorLA + colorLB + colorRB) / 8.0;
	float	bright = pow(gx*gx + gy*gy,0.5);
	vec4	final = color * bright;

	final.rgb = (invert_lines) ? blurred.rgb + final.rgb * intensity : blurred.rgb - final.rgb * intensity;
	final.a = color.a;
	
	gl_FragColor = final;
}