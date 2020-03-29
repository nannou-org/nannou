/*{
    "CATEGORIES": [
        "Color Effect",
        "Utility"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 1,
            "NAME": "r",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 1,
            "NAME": "g",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 1,
            "NAME": "b",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0,
            "NAME": "a",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2"
}
*/

void main() {
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	if (r)
		srcPixel.r = 1.0-srcPixel.r;
	if (g)
		srcPixel.g = 1.0-srcPixel.g;
	if (b)
		srcPixel.b = 1.0-srcPixel.b;
	if (a)
		srcPixel.a = 1.0-srcPixel.a;
	gl_FragColor = srcPixel;
}