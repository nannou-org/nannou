/*{
	"CREDIT": "by Carter Rosenberg",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Color"
	],
	"INPUTS": [
		{
			"NAME": "offset",
			"TYPE": "float",
			"DEFAULT": 0.5
		},
		{
			"NAME": "frequency",
			"TYPE": "float",
			"MAX": 16.0,
			"MIN": 1.0,
			"DEFAULT": 1.0
		},
		{
			"NAME": "curve",
			"TYPE": "long",
			"VALUES": [
				0,
				1,
				2
			],
			"LABELS": [
				"Linear",
				"Sine",
				"Exponential"
			],
			"DEFAULT": 0
		},
		{
			"NAME": "vertical",
			"TYPE": "bool",
			"DEFAULT": 0.0
		},
		{
			"NAME": "startColor",
			"TYPE": "color",
			"DEFAULT": [
				1.0,
				0.75,
				0.0,
				1.0
			]
		},
		{
			"NAME": "endColor",
			"TYPE": "color",
			"DEFAULT": [
				0.0,
				0.25,
				0.75,
				1.0
			]
		}
	]
}*/



const float pi = 3.14159265359;
const float e = 2.71828182846;



void main() {
	float mixAmount = 0.0;
	float phase = offset;
	
	if (vertical)	{
		mixAmount = phase + frequency * isf_FragNormCoord[1];
	}
	else	{
		mixAmount = phase + frequency * isf_FragNormCoord[0];
	}
	
	if (curve == 0)	{
		mixAmount = mod(2.0 * mixAmount,2.0);
		mixAmount = (mixAmount < 1.0) ? mixAmount : 1.0 - (mixAmount - floor(mixAmount));
	}
	else if (curve == 1)	{
		mixAmount = sin(mixAmount * pi * 2.0 - pi / 2.0) * 0.5 + 0.5;
	}
	else if (curve == 2)	{
		mixAmount = mod(2.0 * mixAmount, 2.0);
		mixAmount = (mixAmount < 1.0) ? mixAmount : 1.0 - (mixAmount - floor(mixAmount));
		mixAmount = pow(mixAmount, 2.0);
	}
	
	gl_FragColor = mix(startColor,endColor,mixAmount);
}