/*{
    "CATEGORIES": [
        "Utility"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "Maps the alpha to grayscale",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        }
    ],
    "ISFVSN": "2"
}
*/

void main()	{
	vec4		inputPixelColor = IMG_THIS_PIXEL(inputImage);
	inputPixelColor.rgb = vec3(inputPixelColor.a);
	inputPixelColor.a = 1.0;
	gl_FragColor = inputPixelColor;
}
