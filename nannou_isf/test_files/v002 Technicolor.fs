/*{
    "CATEGORIES": [
        "Stylize",
        "Film",
        "Color Effect",
        "v002"
    ],
    "CREDIT": "by v002",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "amount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABELS": [
                "Strip additive",
                "Strip subtractive",
                "Strip matte"
            ],
            "NAME": "style",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2
            ]
        }
    ],
    "ISFVSN": "2"
}
*/


//	Based on v002 technicolor – https://github.com/v002/v002-Film-Effects/

const vec4 redfilter1 		= vec4(1.0, 0.0, 0.0, 1.0);
const vec4 bluegreenfilter1 	= vec4(0.0, 1.0, 0.7, 1.0);

const vec4 redfilter2 		= vec4(1.0, 0.0, 0.0, 0.0);
const vec4 bluegreenfilter2 	= vec4(0.0, 1.0, 1.0, 0.0);

const vec4 cyanfilter		= vec4(0.0, 1.0, 0.5, 0.0);
const vec4 magentafilter	= vec4(1.0, 0.0, 0.25, 0.0);



void main (void) 
{
	vec4 input0 = IMG_THIS_PIXEL(inputImage);
	vec4 result;

	if (style == 0)	{
		vec4 redrecord = input0 * redfilter1;
		vec4 bluegreenrecord = input0 * bluegreenfilter1;
		vec4 rednegative = vec4(redrecord.r);
		vec4 bluegreennegative = vec4((bluegreenrecord.g + bluegreenrecord.b)/2.0);

		vec4 redoutput = rednegative * redfilter1;
		vec4 bluegreenoutput = bluegreennegative * bluegreenfilter1;

		// additive 'projection"
		result = redoutput + bluegreenoutput;

		result = mix(input0, result, amount);
		result.a = input0.a;
	}
	else if (style == 1)	{
		vec4 redrecord = input0 * redfilter2;
		vec4 bluegreenrecord = input0 * bluegreenfilter2;

		vec4 rednegative = vec4(redrecord.r);
		vec4 bluegreennegative = vec4((bluegreenrecord.g + bluegreenrecord.b)/2.0);

		vec4 redoutput = rednegative + cyanfilter;
		vec4 bluegreenoutput = bluegreennegative + magentafilter;

		result = redoutput * bluegreenoutput;

		result = mix(input0, result, amount);
		result.a = input0.a;	
	}
	else if (style == 2)	{
		vec3 redmatte = vec3(input0.r - ((input0.g + input0.b)/2.0));
		vec3 greenmatte = vec3(input0.g - ((input0.r + input0.b)/2.0));
		vec3 bluematte = vec3(input0.b - ((input0.r + input0.g)/2.0));

		redmatte = 1.0 - redmatte;
		greenmatte = 1.0 - greenmatte;
		bluematte = 1.0 - bluematte;

		vec3 red =  greenmatte * bluematte * input0.r;
		vec3 green = redmatte * bluematte * input0.g;
		vec3 blue = redmatte * greenmatte * input0.b;

		result = vec4(red.r, green.g, blue.b, input0.a);

		result = mix(input0, result, amount);
		result.a = input0.a;	
	}
	gl_FragColor = result;		
} 