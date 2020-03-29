/*{
    "CATEGORIES": [
        "Geometry Adjustment"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "Moves the vertex points to the specified locations without correction",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": [
                0,
                1
            ],
            "LABEL": "Top Left",
            "NAME": "topleft",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": [
                0,
                0
            ],
            "LABEL": "Bottom Left",
            "NAME": "bottomleft",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": [
                1,
                1
            ],
            "LABEL": "Top Right",
            "NAME": "topright",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": [
                1,
                0
            ],
            "LABEL": "Bottom Right",
            "NAME": "bottomright",
            "TYPE": "point2D"
        }
    ],
    "ISFVSN": "2"
}
*/


void main()
{
	vec4		test = IMG_THIS_PIXEL(inputImage);
	gl_FragColor = test;
}
