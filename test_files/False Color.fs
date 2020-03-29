/*{
	"CREDIT": "by zoidberg",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Color Effect"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "brightColor",
			"TYPE": "color",
			"DEFAULT": [
				1.0,
				0.9,
				0.8,
				1.0
			]
		},
		{
			"NAME": "darkColor",
			"TYPE": "color",
			"DEFAULT": [
				0.3,
				0.0,
				0.0,
				1.0
			]
		}
	]
}*/

//const vec4		lumcoeff = vec4(0.299, 0.587, 0.114, 0.0);
const vec4 	lumcoeff = vec4(0.2126, 0.7152, 0.0722, 0.0);

void main() {
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	float		luminance = dot(srcPixel,lumcoeff);
	gl_FragColor = mix(vec4(darkColor.rgb, srcPixel.a), vec4(brightColor.rgb, srcPixel.a), luminance);
}
