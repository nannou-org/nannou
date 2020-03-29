/*{
    "CATEGORIES": [
        "Retro",
        "Stylize",
        "Color Effect"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "Posterizes an image",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 1.25,
            "LABEL": "Gamma",
            "MAX": 2,
            "MIN": 0.5,
            "NAME": "gamma",
            "TYPE": "float"
        },
        {
            "DEFAULT": 6,
            "LABEL": "Quality",
            "MAX": 32,
            "MIN": 3,
            "NAME": "numColors",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

void main()	{
	
    vec4	c = IMG_THIS_PIXEL(inputImage);

 	c.rgb = pow(c.rgb, vec3(gamma, gamma, gamma));
 	c.rgb = c.rgb * numColors;
 	c.rgb = floor(c.rgb);
 	c.rgb = c.rgb / numColors;
 	c.rgb = pow(c.rgb, vec3(1.0/gamma));

	gl_FragColor = c;
}
