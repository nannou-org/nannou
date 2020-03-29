/*{
    "CATEGORIES": [
        "Audio Visualizer",
        "Distortion Effect"
    ],
    "CREDIT": "icalvin102 (calvin@icalvin.de)",
    "DESCRIPTION": "Displaces image with audio waveform",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "LABEL": "Audio Waveform",
            "NAME": "audio",
            "TYPE": "audio"
        },
        {
            "DEFAULT": 0.1,
            "LABEL": "Displace X",
            "MAX": 1,
            "MIN": 0,
            "NAME": "displaceX",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.1,
            "LABEL": "Displace Y",
            "MAX": 1,
            "MIN": 0,
            "NAME": "displaceY",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "Detail X",
            "MAX": 1,
            "MIN": 0,
            "NAME": "detailX",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "LABEL": "Detail Y",
            "MAX": 1,
            "MIN": 0,
            "NAME": "detailY",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "DESCRIPTION": "Renderpass 0"
        }
    ]
}
*/

void main()	{

	vec4 inputPixelColor;
	vec2 uv = isf_FragNormCoord.xy;
	vec2 waveLoc = fract((uv)*vec2(detailX, detailY));

	vec2 wave = vec2(IMG_NORM_PIXEL(audio, vec2(waveLoc.x, 0.0)).r, IMG_NORM_PIXEL(audio, vec2(waveLoc.y, 1.0)).r)-.5;
	wave *= vec2(displaceY, displaceX);

	inputPixelColor = IMG_NORM_PIXEL(inputImage, uv + wave.yx);

	gl_FragColor = inputPixelColor;
}
