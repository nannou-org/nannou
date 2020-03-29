/*{
	"CREDIT": "by zoidberg",
	"ISFVSN": "2",
	"DESCRIPTION": "Modifies the white point by multiplying the src pixel by the color value",
	"CATEGORIES": [
		"Color Adjustment"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "newWhite",
			"TYPE": "color",
			"DEFAULT": [
				1.0,
				1.0,
				1.0,
				1.0
			]
		}
	]
}*/



void main() {
	vec4		tmpColorA = IMG_THIS_PIXEL(inputImage);
	gl_FragColor = tmpColorA * newWhite;
}