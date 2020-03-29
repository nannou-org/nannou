/*{
    "CATEGORIES": [
        "Feedback"
    ],
    "CREDIT": "by VIDVOX",
    "DESCRIPTION": "Pixel with brightness levels below the threshold do not update.",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Threshold",
            "MAX": 1,
            "MIN": 0,
            "NAME": "thresh",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Gain",
            "MAX": 2,
            "MIN": 0,
            "NAME": "gain",
            "TYPE": "float"
        },
        {
            "DEFAULT": true,
            "LABEL": "Hard Cutoff",
            "NAME": "hardCutoff",
            "TYPE": "bool"
        },
        {
            "DEFAULT": false,
            "LABEL": "Invert",
            "NAME": "invert",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "FLOAT": true,
            "PERSISTENT": true,
            "TARGET": "bufferVariableNameA"
        },
        {
        }
    ]
}
*/

void main()
{
	vec4		freshPixel = IMG_PIXEL(inputImage,gl_FragCoord.xy);
	vec4		stalePixel = IMG_PIXEL(bufferVariableNameA,gl_FragCoord.xy);
	float		brightLevel = (freshPixel.r + freshPixel.b + freshPixel.g) / 3.0;
	if (invert)
		brightLevel = 1.0 - brightLevel;
	brightLevel = brightLevel * gain;
	if (hardCutoff)	{
		if (brightLevel < thresh)
			brightLevel = 1.0;
		else
			brightLevel = 0.0;
	}
	gl_FragColor = mix(freshPixel,stalePixel, brightLevel);
}
