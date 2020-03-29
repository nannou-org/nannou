/*{
    "CATEGORIES": [
        "Color Effect",
        "Retro"
    ],
    "CREDIT": "by zoidberg",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 30,
            "MAX": 30,
            "MIN": 2,
            "NAME": "levels",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

void main() {
	//	get the src pixel, convert to HSL, posterize the 'L', convert back to RGB
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	vec4		amountPerLevel = vec4(1.0/levels);
	vec4		numOfLevels = floor(srcPixel/amountPerLevel);
	vec4		outColor = numOfLevels * (vec4(1.0) / (vec4(levels) - vec4(1.0)));
	outColor.a = srcPixel.a;
	gl_FragColor = outColor;
}
