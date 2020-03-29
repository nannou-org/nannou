/*{
	"CREDIT": "by zoidberg",
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
			"NAME": "bright",
			"TYPE": "float",
			"MIN": -1.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		}
	]
}*/

void main() {
	gl_FragColor = clamp(IMG_THIS_PIXEL(inputImage) + vec4(bright,bright,bright,0.0), 0.0, 1.0);
}