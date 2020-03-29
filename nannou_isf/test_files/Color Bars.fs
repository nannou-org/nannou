/*{
	"DESCRIPTION": "",
	"CREDIT": "VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Utility"
	],
	"INPUTS": [
		{
			"NAME": "colorShift",
			"LABEL": "Color Shift",
			"TYPE": "float",
			"DEFAULT": 0.0,
			"MIN": 0.0,
			"MAX": 1.0
		}
	]
	
}*/




vec4 colorsArray[9];




void main()	{	
	vec4		outputPixelColor = vec4(0.0);
	vec2		loc = isf_FragNormCoord.xy;
	
	//	figure out if we are in the top, middle or bottom sections
	//	these are broken into the ratios 3/4, 1/8, 1/8
	
	//	if we are in the top section figure out which of the 9 colors to use
	if (loc.y > 0.25)	{
		colorsArray[0] = vec4(0.412, 0.412, 0.412, 1.0);
		colorsArray[1] = vec4(0.757, 0.757, 0.757, 1.0);
		colorsArray[2] = vec4(0.757, 0.757, 0.000, 1.0);
		colorsArray[3] = vec4(0.000, 0.757, 0.757, 1.0);
		colorsArray[4] = vec4(0.000, 0.757, 0.000, 1.0);
		colorsArray[5] = vec4(0.757, 0.000, 0.757, 1.0);
		colorsArray[6] = vec4(0.757, 0.000, 0.000, 1.0);
		colorsArray[7] = vec4(0.000, 0.000, 0.757, 1.0);
		colorsArray[8] = vec4(0.412, 0.412, 0.412, 1.0);
		
		int		colorIndex = int((9.0 * mod(loc.x + colorShift, 1.0)));
		
		outputPixelColor = colorsArray[colorIndex];
	}
	//	in the 'middle section we draw the black to white image
	else if (loc.y > 0.125)	{
		outputPixelColor.rgb = vec3(loc.x);
		outputPixelColor.a = 1.0;
	}
	else	{
		colorsArray[0] = vec4(0.169, 0.169, 0.169, 1.0);
		colorsArray[1] = vec4(0.019, 0.019, 0.019, 1.0);
		colorsArray[2] = vec4(1.000, 1.000, 1.000, 1.0);
		colorsArray[3] = vec4(1.000, 1.000, 1.000, 1.0);
		colorsArray[4] = vec4(0.019, 0.019, 0.019, 1.0);
		colorsArray[5] = vec4(0.000, 0.000, 0.000, 1.0);
		colorsArray[6] = vec4(0.019, 0.019, 0.019, 1.0);
		colorsArray[7] = vec4(0.038, 0.038, 0.038, 1.0);
		colorsArray[8] = vec4(0.169, 0.169, 0.169, 1.0);
		
		int		colorIndex = int((9.0 * loc.x));
		
		outputPixelColor = colorsArray[colorIndex];		
	}
	
	gl_FragColor = outputPixelColor;
}
