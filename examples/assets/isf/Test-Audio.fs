/*
{
  "CATEGORIES" : [
    "Generator"
  ],
  "DESCRIPTION" : "Visualizes an FFT analysis image with custom set colors for frequency domain",
  "INPUTS" : [
    {
      "NAME" : "waveImage",
      "TYPE" : "audio"
    },
    {
      "NAME" : "waveSize",
      "TYPE" : "float",
      "MAX" : 0.5,
      "DEFAULT" : 0.05,
      "MIN" : 0
    },
    {
      "NAME" : "stereo",
      "TYPE" : "bool",
      "DEFAULT" : 1
    }
  ],
  "CREDIT" : "by VIDVOX"
}
*/



void main() {
	
	vec2		loc = vec2(vv_FragNormCoord[1], vv_FragNormCoord[0]);
	
	vec2		rawSize = IMG_SIZE(waveImage);
	float		channel = 0.5;
	float		offset = 0.0;
	if (stereo == true)	{
		channel = (loc.x > 0.5) ? 0.0 : 1.0;
		offset = (loc.x > 0.5) ? 0.25 : -0.25;
	}
	
	vec2		waveLoc = vec2(loc.y,channel);
	vec4		wave = IMG_NORM_PIXEL(waveImage, waveLoc)+offset;
	vec4		waveAdd = (1.0 - smoothstep(0.0, waveSize, abs(wave - loc.x)));
	waveAdd.a = 1.0;
	gl_FragColor = waveAdd;
}