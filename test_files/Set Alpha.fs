/*{
    "CATEGORIES": [
        "Color Adjustment",
        "Utility"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "Sets the alpha channel of the image",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "New Alpha",
            "MAX": 1,
            "MIN": 0,
            "NAME": "newAlpha",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

void main()	{
	vec4		inputPixelColor = IMG_THIS_NORM_PIXEL(inputImage);
	
	inputPixelColor.a = newAlpha;
	
	gl_FragColor = inputPixelColor;
}
