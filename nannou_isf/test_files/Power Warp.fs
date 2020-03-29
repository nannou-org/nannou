/*{
    "CATEGORIES": [
        "Distortion Effect"
    ],
    "CREDIT": "",
    "DESCRIPTION": "Power curves distortions with shifting",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 1,
            "MAX": 4,
            "MIN": 0.25,
            "NAME": "power_x",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 4,
            "MIN": 0.25,
            "NAME": "power_y",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "shift_x",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "shift_y",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABELS": [
                "Style 1",
                "Style 2"
            ],
            "NAME": "mode_x",
            "TYPE": "long",
            "VALUES": [
                0,
                1
            ]
        },
        {
            "DEFAULT": 1,
            "LABELS": [
                "Style 1",
                "Style 2"
            ],
            "NAME": "mode_y",
            "TYPE": "long",
            "VALUES": [
                0,
                1
            ]
        }
    ],
    "ISFVSN": "2"
}
*/


const float pi = 3.14159265359;



void main()	{
	vec4		inputPixelColor;
	vec2		pos = isf_FragNormCoord.xy;
	
	if (mode_x == 0)	{
		pos.x = pow(pos.x, power_x);
	}
	else	{
		if (pos.x > 0.5)
			pos.x = 0.5 + pow(2.0*(pos.x - 0.5), power_x) / 2.0;
		else	{
			pos.x = pow(1.0 - 2.0*pos.x, power_x) / 2.0;
			pos.x = 0.5 - pos.x;
		}
	}
	pos.x = mod(pos.x + shift_x, 1.0);
	
	if (mode_y == 0)	{
		pos.y = pow(pos.y, power_y);
	}
	else	{
		if (pos.y > 0.5)
			pos.y = 0.5 + pow(2.0*(pos.y - 0.5), power_y) / 2.0;
		else	{
			pos.y = pow(1.0 - 2.0*pos.y, power_y) / 2.0;
			pos.y = 0.5 - pos.y;
		}
	}
	pos.y = mod(pos.y + shift_y, 1.0);
	
	inputPixelColor = IMG_NORM_PIXEL(inputImage, pos);
	
	gl_FragColor = inputPixelColor;
}
