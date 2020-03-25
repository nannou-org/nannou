/*{
	"DESCRIPTION": "",
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Glitch", "Color Effect"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "horizontal_magnitude",
			"TYPE": "float",
			"MIN": 0.00,
			"MAX": 1.0,
			"DEFAULT": 0.125
		},
		{
			"NAME": "vertical_magnitude",
			"TYPE": "float",
			"MIN": 0.00,
			"MAX": 1.0,
			"DEFAULT": 0.125
		},
		{
			"NAME": "color_magnitude",
			"TYPE": "float",
			"MIN": 0.00,
			"MAX": 2.0,
			"DEFAULT": 1.0
		},
		{
			"NAME": "mode",
			"VALUES": [
				0,
				1,
				2,
				3
			],
			"LABELS": [
				"add",
				"add mod",
				"multiply",
				"difference"
			],
			"DEFAULT": 0,
			"TYPE": "long"
		}
	]
	
}*/



//	adapted from maxilla inc's https://github.com/maxillacult/ofxPostGlitch/



float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}



void main()
{
	
	vec2 texCoord = isf_FragNormCoord;

	vec4 col = IMG_NORM_PIXEL(inputImage,texCoord);

	vec4 col_r = vec4(0.0);
	vec4 col_l = vec4(0.0);
	vec4 col_g = vec4(0.0);

	vec2 rand_offset = texCoord + vec2((horizontal_magnitude * rand(vec2(TIME,0.213))-horizontal_magnitude/2.0), (vertical_magnitude * rand(vec2(TIME,0.463467))) - vertical_magnitude / 2.0);
	col_r = IMG_NORM_PIXEL(inputImage,rand_offset);
	rand_offset = texCoord + vec2((horizontal_magnitude * rand(vec2(TIME,0.5345))-horizontal_magnitude/2.0), (vertical_magnitude * rand(vec2(TIME,0.7875))) - vertical_magnitude / 2.0);
	col_l = IMG_NORM_PIXEL(inputImage,rand_offset);
	rand_offset = texCoord + vec2((horizontal_magnitude * rand(vec2(TIME,0.456345))-horizontal_magnitude/2.0), (vertical_magnitude * rand(vec2(TIME,0.9432))) - vertical_magnitude / 2.0);
	col_g = IMG_NORM_PIXEL(inputImage,rand_offset);

	vec4 color_shift;
	color_shift.b = color_magnitude*col_r.b*max(1.0,sin(texCoord.y*1.2)*2.5)*rand(vec2(TIME,0.0342));
	color_shift.r = color_magnitude*col_l.r*max(1.0,sin(texCoord.y*1.2)*2.5)*rand(vec2(TIME,0.5253));
	color_shift.g = color_magnitude*col_g.g*max(1.0,sin(texCoord.y*1.2)*2.5)*rand(vec2(TIME,0.1943));

	//	if doing add maths
	if (mode == 0)	{
		col = col + color_shift;
	}
	//	add mod
	else if (mode == 1)	{
		col = mod(col + color_shift,1.001);
	}
	//	multiply
	else if (mode == 2)	{
		col = mix(col, col * (color_magnitude + color_shift),0.9);
	}
	// difference
	else if (mode == 3)	{
		col = abs(color_shift - col);
	}

	gl_FragColor = col;
}
