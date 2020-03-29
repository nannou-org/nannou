/*{
    "CATEGORIES": [
        "Color Effect",
        "Utility"
    ],
    "CREDIT": "by zoidberg",
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
	float		minComponent = min(srcPixel.r, min(srcPixel.g, srcPixel.b));
	gl_FragColor = vec4(minComponent, minComponent, minComponent, srcPixel.a);
}