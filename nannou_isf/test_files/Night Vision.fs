/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Stylize"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "luminanceThreshold",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.2
		},
		{
			"NAME": "colorAmplification",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 10.0,
			"DEFAULT": 4.0
		},
		{
			"NAME": "noiseLevel",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.1
		},
		{
			"NAME": "visionColor",
			"TYPE": "color",
			"DEFAULT": [
				0.1,
				0.95,
				0.2,
				1.0
			]
		}
	],
	"IMPORTED": {
		"noiseTex": {
			"PATH": "noise2d.png"
		}
	}
}*/



void main ()	{
	vec2		normSrcCoord;

	normSrcCoord.x = isf_FragNormCoord[0];
	normSrcCoord.y = isf_FragNormCoord[1];
	
	vec4 finalColor;
	
	// Set effectCoverage to 1.0 for normal use.  

	vec2 uv;           
	uv.x = 0.4*sin(TIME*50.0);                                 
	uv.y = 0.4*cos(TIME*50.0);                                 
	vec3 n = IMG_NORM_PIXEL(noiseTex,mod(normSrcCoord.xy*3.5 + uv,1.0)).rgb;
	vec3 c = IMG_THIS_PIXEL(inputImage).rgb + n.rgb * noiseLevel;
	
	const vec4 	lumacoeff = vec4(0.2126, 0.7152, 0.0722, 0.0);
	float		lum = dot(vec4(c,1.), lumacoeff);
	if (lum < luminanceThreshold)	{
	  c *= colorAmplification; 
	}
	finalColor.rgb = (c + (n*0.2)) * visionColor.rgb;

	gl_FragColor.rgb = finalColor.rgb;
	gl_FragColor.a = 1.0;
}