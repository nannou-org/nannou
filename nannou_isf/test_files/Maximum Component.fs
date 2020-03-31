/*{
	"CREDIT": "by zoidberg",
	"ISFVSN": "2",
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
	float		maxComponent = max(srcPixel.r, max(srcPixel.g, srcPixel.b));
	gl_FragColor = vec4(maxComponent, maxComponent, maxComponent, srcPixel.a);
}