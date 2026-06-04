/*{
	"DESCRIPTION": "Visualizes an FFT analysis image with custom set colors for frequency domain",
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Generator"
	],
	"INPUTS": [
		{
			"NAME": "fftImage",
			"TYPE": "audioFFT"
		},
		{
			"NAME": "strokeSize",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 0.25,
			"DEFAULT": 0.01
		},
		{
			"NAME": "gain",
			"TYPE": "float",
			"MIN": 1.0,
			"MAX": 5.0,
			"DEFAULT": 1.0
		},
		{
			"NAME": "minRange",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "maxRange",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.9
		},
		{
			"NAME": "topColor",
			"TYPE": "color",
			"DEFAULT": [
				0.0,
				0.0,
				0.0,
				0.0
			]
		},
		{
			"NAME": "bottomColor",
			"TYPE": "color",
			"DEFAULT": [
				0.0,
				0.5,
				0.9,
				1.0
			]
		},
		{
			"NAME": "strokeColor",
			"TYPE": "color",
			"DEFAULT": [
				0.25,
				0.25,
				0.25,
				1.0
			]
		}
	]
}*/



void main() {
	
	vec2 loc = isf_FragNormCoord;
	
	//	the fftImage is 256 steps
	loc.x = loc.x * abs(maxRange - minRange) + minRange;
	
	vec4 fft = IMG_NORM_PIXEL(fftImage, vec2(loc.x,0.5));
	float fftVal = gain * (fft.r + fft.g + fft.b) / 3.0;
	if (loc.y > fftVal)
		fft = topColor;
	else
		fft = bottomColor;
	if ((strokeSize > 0.0) && (abs(fftVal - loc.y) < strokeSize))	{
		fft = mix(strokeColor, fft, abs(fftVal - loc.y) / strokeSize);
	}
	
	//(smoothstep(0.0, stroke, abs(fftVal - loc.y))) * strokeColor);
	gl_FragColor = fft;
}