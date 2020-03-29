/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Color Adjustment"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "minLevel",
			"LABEL": "Minimum Point",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.2
		},
		{
			"NAME": "midLevel",
			"LABEL": "Mid Point",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.5
		},
		{
			"NAME": "maxLevel",
			"LABEL": "Maximum Point",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.8
		},
		{
			"NAME": "offset1",
			"TYPE": "color",
			"DEFAULT": [
				0.5,
				0.5,
				0.5,
				0.5
			]
		},
		{
			"NAME": "offset2",
			"TYPE": "color",
			"DEFAULT": [
				0.5,
				0.5,
				0.5,
				0.5
			]
		},
		{
			"NAME": "offset3",
			"TYPE": "color",
			"DEFAULT": [
				0.5,
				0.5,
				0.5,
				0.5
			]
		},
		{
			"NAME": "offset4",
			"TYPE": "color",
			"DEFAULT": [
				0.5,
				0.5,
				0.5,
				0.5
			]
		},
		{
			"NAME": "offset5",
			"TYPE": "color",
			"DEFAULT": [
				0.5,
				0.5,
				0.5,
				0.5
			]
		},
		{
			"NAME": "offset6",
			"TYPE": "color",
			"DEFAULT": [
				0.5,
				0.5,
				0.5,
				0.5
			]
		},
		{
			"NAME": "levelsMode",
			"LABEL": "Levels Mode",
			"TYPE": "bool",
			"DEFAULT": 0.0
		}
	]
}*/



void main() {
	vec4		tmpColor = IMG_THIS_PIXEL(inputImage);
	float brightness = (tmpColor.r + tmpColor.g + tmpColor.b) * tmpColor.a / 3.0;
	
	//	all adjustments
	if (brightness <= minLevel)	{
		tmpColor = tmpColor + offset1 - 0.5;
	}
	else if (brightness <= (minLevel + midLevel)/2.0)	{
		tmpColor = tmpColor + offset2 - 0.5;
	}
	else if (brightness <= midLevel)	{
		tmpColor = tmpColor + offset3 - 0.5;
	}
	else if (brightness <= (maxLevel + midLevel)/2.0)	{
		tmpColor = tmpColor + offset4 - 0.5;
	}
	else if (brightness <= maxLevel)	{
		tmpColor = tmpColor + offset5 - 0.5;
	}
	else	{
		tmpColor = tmpColor + offset6 - 0.5;
	}	
	
	if (levelsMode)	{
		//	all adjustments
		tmpColor.rgb = vec3(1.0);
		
		if (brightness <= minLevel)	{
			tmpColor.a = 0.0;
		}
		else if (brightness <= (minLevel + midLevel)/2.0)	{
			tmpColor.a = 1.0/5.0;
		}
		else if (brightness <= midLevel)	{
			tmpColor.a = 2.0/5.0;
		}
		else if (brightness <= (maxLevel + midLevel)/2.0)	{
			tmpColor.a = 3.0/5.0;
		}
		else if (brightness <= maxLevel)	{
			tmpColor.a = 4.0/5.0;
		}
		else	{
			tmpColor.a = 1.0;
		}
	}
	gl_FragColor = clamp(tmpColor, 0.0, 1.0);
}

