/*{
    "CATEGORIES": [
        "Color Adjustment",
        "Masking",
        "Utility"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        }
    ],
    "ISFVSN": "2"
}
*/

void main() {
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	srcPixel.rgb = srcPixel.rgb * srcPixel.a;
	srcPixel.a = 1.0;
	gl_FragColor = srcPixel;
}