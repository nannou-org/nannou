/*{
	"CREDIT": "by v002",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Halftone Effect", "v002"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "invert",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "separation",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 0.5,
			"DEFAULT": 0.25
		},
		{
			"NAME": "greyscale",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.75
		},
		{
			"NAME": "thickness",
			"TYPE": "float",
			"MIN": 0.5,
			"MAX": 1.0,
			"DEFAULT": 0.75
		},
		{
			"NAME": "front",
			"TYPE": "color",
			"DEFAULT": [
				0.5,
				0.0,
				0.25,
				1.0
			]
		},
		{
			"NAME": "back",
			"TYPE": "color",
			"DEFAULT": [
				0.2,
				0.75,
				0.5,
				1.0
			]
		}
	]
}*/


//Original Source
//http://learningwebgl.com/blog/?p=2858

//	Based on https://github.com/v002/v002-Half-Tones/




const vec4 lumacoeff = vec4(0.2126, 0.7152, 0.0722, 0.0);


void main()
{
    vec4 color = IMG_THIS_PIXEL(inputImage);
    float lum = dot(color, lumacoeff);
    vec2 coord = gl_FragCoord.xy;
    
    vec4 colora = mix(front, back, invert);
    vec4 colorb = mix(back, front, invert);
    
    colora.a *= color.a;
    colorb.a *= color.a;
    
    vec4 cout = mix(color,colora, greyscale);

    gl_FragColor = colorb;
    
    if (lum > 1.00)
    {
        if (mod(coord.x + coord.y, thickness) >= separation)
            gl_FragColor = cout;
    }
    
    if (lum > 0.75)
    {
        if (mod(coord.x - coord.y, thickness) >= separation )
            gl_FragColor = cout;
    }
    
    if (lum > 0.50)
    {
        if (mod(coord.x + coord.y - (thickness * 0.5), thickness) >= separation)
            gl_FragColor = cout;
    }
    
    if (lum > 0.3)
    {
        if (mod(coord.x - coord.y - (thickness * 0.5), thickness) >= separation)
            gl_FragColor = cout;
    }
}