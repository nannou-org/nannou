/*{
    "CATEGORIES": [
        "Audio Visualizer"
    ],
    "CREDIT": "icalvin102 (calvin@icalvin.de)",
    "DESCRIPTION": "Generates a radial audiospectrogram from fft-input",
    "INPUTS": [
        {
            "LABEL": "AudioFFT",
            "NAME": "audioFFT",
            "TYPE": "audioFFT"
        },
        {
            "DEFAULT": 0.001,
            "LABEL": "Size",
            "MAX": 0.05,
            "MIN": 1,
            "NAME": "size",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.99,
            "LABEL": "Feedback Opacity",
            "MAX": 1,
            "MIN": 0.7,
            "NAME": "feedbackOpacity",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Lowest Frequency",
            "MAX": 1,
            "MIN": 0,
            "NAME": "low",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Heighest Frequency",
            "MAX": 1,
            "MIN": 0,
            "NAME": "high",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "FLOAT": true,
            "HEIGHT": "1",
            "PERSISTENT": true,
            "TARGET": "time",
            "WIDTH": "1"
        },
        {
            "FLOAT": true,
            "PERSISTENT": true,
            "TARGET": "buff"
        },
        {
        }
    ]
}
*/

void main()	{
	vec4 inputPixelColor = IMG_THIS_PIXEL(buff);
	float t = IMG_PIXEL(time, vec2(0.0,0.0)).r;
	if(PASSINDEX == 0){
		gl_FragColor = vec4(t+size);
		return;
	}
	if(PASSINDEX == 1){
		vec2 dir = vec2(sin(t), cos(t));
		vec2 co = (isf_FragNormCoord - vec2(0.5,0.5)) * 2.0;
 		float radius=length(co);
 		inputPixelColor *= feedbackOpacity;
 		if( dot(dir, normalize(co)) >= cos(size) && radius <= 1.0){
			inputPixelColor = IMG_NORM_PIXEL(audioFFT, vec2((1.0-radius) * abs(high - low) + low, .5));
 		}
 		inputPixelColor.a = 1.0;
	}
	gl_FragColor = inputPixelColor;
}
