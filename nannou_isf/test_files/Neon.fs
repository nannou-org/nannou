/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"DESCRIPTION": "adapted from https://github.com/neilmendoza/ofxPostProcessing/blob/master/src/GodRaysPass.cpp",
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
			"MAX": 10.0,
			"DEFAULT": 5.0
		},
		{
			"NAME": "gain",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 1.0
		},
		{
			"NAME": "neonColor",
			"TYPE": "color",
			"DEFAULT": [
				1.0,
				0.4,
				0.64,
				1.0
			]
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


void main(void)
{

	//	edges // rays // color
	vec4 color = IMG_THIS_PIXEL(inputImage);
	vec4 colorL = IMG_NORM_PIXEL(inputImage, left_coord);
	vec4 colorR = IMG_NORM_PIXEL(inputImage, right_coord);
	vec4 colorA = IMG_NORM_PIXEL(inputImage, above_coord);
	vec4 colorB = IMG_NORM_PIXEL(inputImage, below_coord);

	vec4 colorLA = IMG_NORM_PIXEL(inputImage, lefta_coord);
	vec4 colorRA = IMG_NORM_PIXEL(inputImage, righta_coord);
	vec4 colorLB = IMG_NORM_PIXEL(inputImage, leftb_coord);
	vec4 colorRB = IMG_NORM_PIXEL(inputImage, rightb_coord);

	float gx = (0.0);
	float gy = (0.0);
	gx = (-1.0 * gray(colorLA)) + (-1.0 * gray(colorL)) + (-1.0 * gray(colorLB)) + (1.0 * gray(colorRA)) + (1.0 * gray(colorR)) + (1.0 * gray(colorRB));
	gy = (1.0 * gray(colorLA)) + (1.0 * gray(colorA)) + (1.0 * gray(colorRA)) + (-1.0 * gray(colorRB)) + (-1.0 * gray(colorB)) + (-1.0 * gray(colorLB));



	float bright = pow(gx*gx + gy*gy,0.5);
	vec4 final = color * bright;
	
	//	if the brightness is below the threshold draw black
	if (bright < 0.01)	{
		final = vec4(0.0);
	}
	else	{
		final = final * intensity;
	}
	
	gl_FragColor = final;

	vec4 origColor = final;
	vec4 raysColor = color;
	int NUM_SAMPLES = 30;

	float exposure	= 0.1/float(NUM_SAMPLES);
	float decay		= 1.0 ;
	float density	= 0.5;
	float weight	= 6.0;
	float illuminationDecay = 1.0;
	vec2		normSrcCoord;

	normSrcCoord.x = isf_FragNormCoord[0];
	normSrcCoord.y = isf_FragNormCoord[1];

	vec2 deltaTextCoord = vec2(normSrcCoord.st - 0.5);
	vec2 textCoo = normSrcCoord;
	deltaTextCoord *= 1.0 / float(NUM_SAMPLES) * density;

	for(int i=0; i < NUM_SAMPLES ; i++)	{
		textCoo -= deltaTextCoord;
		vec4 tsample = IMG_NORM_PIXEL(inputImage, textCoo);
		tsample *= illuminationDecay * weight;
		raysColor += tsample;
		illuminationDecay *= decay;
	}
	raysColor *= exposure * gain;
	float p = 0.3 *raysColor.g + 0.59*raysColor.r + 0.11*raysColor.b;
	
	gl_FragColor = gray(origColor + p) * neonColor;
	
}
