/*{
	"CATEGORIES" : [
		"Histogram", "Utility"
  	],
	"DESCRIPTION": "Draws an RGB histogram from a provided histogram image",
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"INPUTS": [
		{
			"NAME": "histogramImage",
			"TYPE": "image"
		}
	]
}*/


void main()	{
	vec4        outColor = vec4(0., 0., 0., 0.);
	vec4        histoVals = IMG_NORM_PIXEL(histogramImage, vec2(isf_FragNormCoord.x, 0.5));
	if (histoVals.r >= isf_FragNormCoord.y)	{
		outColor.r = 1.0;
		outColor.a = 1.0;
	}
	if (histoVals.g >= isf_FragNormCoord.y)	{
		outColor.g = 1.0;
		outColor.a = 1.0;
	}
	if (histoVals.b >= isf_FragNormCoord.y)	{
		outColor.b = 1.0;
		outColor.a = 1.0;
	}
	
	gl_FragColor = outColor;
	
}
