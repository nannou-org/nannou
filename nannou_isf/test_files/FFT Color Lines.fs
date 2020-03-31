/*
{
  "CATEGORIES" : [
    "Audio Visualizer"
  ],
  "ISFVSN": "2",
  "DESCRIPTION" : "Visualizes an FFT analysis image with custom set colors for frequency domain",
  "INPUTS" : [
    {
      "NAME" : "fftImage",
      "TYPE" : "audioFFT"
    },
    {
      "NAME" : "waveImage",
      "TYPE" : "audio"
    },
    {
      "NAME" : "gainFFT",
      "TYPE" : "float",
      "MAX" : 5,
      "DEFAULT" : 1,
      "MIN" : 0
    },
    {
      "NAME" : "rangeFFT",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.9,
      "MIN" : 0
    },
    {
      "NAME" : "waveSize",
      "TYPE" : "float",
      "MAX" : 0.5,
      "DEFAULT" : 0.05,
      "MIN" : 0
    },
    {
      "NAME" : "vertical",
      "TYPE" : "bool",
      "DEFAULT" : 1
    },
    {
      "NAME" : "stereo",
      "TYPE" : "bool",
      "DEFAULT" : 1
    },
    {
      "NAME" : "color1",
      "TYPE" : "color",
      "DEFAULT" : [
        0.2087394440714313,
        0.9861069917678833,
        0.1871179742814854,
        1
      ]
    },
    {
      "NAME" : "color2",
      "TYPE" : "color",
      "DEFAULT" : [
        0,
        0.5,
        1,
        1
      ]
    },
    {
      "NAME" : "color3",
      "TYPE" : "color",
      "DEFAULT" : [
        0,
        1,
        0,
        1
      ]
    },
    {
      "NAME" : "wavecolor",
      "TYPE" : "color",
      "DEFAULT" : [
        1,
        1,
        1,
        1
      ]
    }
  ],
  "CREDIT" : "by VIDVOX"
}
*/



void main() {
	
	vec2		loc = isf_FragNormCoord;

	if (vertical)	{
		loc.x = isf_FragNormCoord[1];
		loc.y = isf_FragNormCoord[0];
	}
	
	vec4 mixColor = color1;
	
	if (loc.y > 0.5)	{
		mixColor = mix (color2,color3,(loc.y-0.5)*2.0);
	}
	else	{
		mixColor = mix (color1,color2,(loc.y*2.0));
	}
	
	loc.y = loc.y * rangeFFT;
	
	vec2		fftSize = IMG_SIZE(fftImage);
	vec2		rawSize = IMG_SIZE(waveImage);
	float		channel = 0.5;
	float		offset = 0.0;
	if (stereo == true)	{
		channel = (loc.x > 0.5) ? 0.0 : 1.0;
		offset = (loc.x > 0.5) ? 0.25 : -0.25;
	}
	
	vec4 		fft = IMG_NORM_PIXEL(fftImage, vec2(loc.y,channel));
	fft = mixColor * fft.r * 3.0;
	fft.rgb = gainFFT * fft.rgb;
	vec2		waveLoc = vec2(loc.y,channel);
	vec4		wave = IMG_NORM_PIXEL(waveImage, waveLoc)+offset;
	//wave = vec4(wave.r);
	vec4		waveAdd = ((1.0 - smoothstep(0.0, waveSize, abs(wave - loc.x))) * wavecolor) * wavecolor.a;
	fft += waveAdd;
	fft.a = mixColor.a + clamp((waveAdd.r + waveAdd.g + waveAdd.b) * wavecolor.a,0.0,1.0);
	
	gl_FragColor = fft;
}