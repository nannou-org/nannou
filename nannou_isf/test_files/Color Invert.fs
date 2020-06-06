/*{
	"CREDIT": "by zoidberg",
	"ISFVSN": "2",
	"DESCRIPTION": "Inverts the RGB channels of the input",
	"CATEGORIES": [
		"Color Effect", "Utility"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		}
	]
}*/

void main() {
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	gl_FragColor = vec4(1.0-srcPixel.r, 1.0-srcPixel.g, 1.0-srcPixel.b, srcPixel.a);
}