/*{
	"DESCRIPTION": "Performs hue shifts of different amounts depending on brightness levels",
	"CREDIT": "VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Color Effect"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "shiftLow",
			"TYPE": "float",
			"DEFAULT": 0.0,
			"MIN": 0.0,
			"MAX": 1.0
		},
		{
			"NAME": "shiftMid",
			"TYPE": "float",
			"DEFAULT": 0.0,
			"MIN": 0.0,
			"MAX": 1.0
		},
		{
			"NAME": "shiftHigh",
			"TYPE": "float",
			"DEFAULT": 0.0,
			"MIN": 0.0,
			"MAX": 1.0
		}
	]
	
}*/



vec3 rgb2hsv(vec3 c)	{
	vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	//vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
	//vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
	vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
	vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);
	
	float d = q.x - min(q.w, q.y);
	float e = 1.0e-10;
	return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)	{
	vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
	vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
	return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}



void main()	{
	vec4		inputPixelColor = IMG_THIS_PIXEL(inputImage);

	//	don't bother doing anything if we're not shifting anything
	if ((shiftLow > 0.0)||(shiftMid > 0.0)||(shiftHigh > 0.0))	{
		//	what is the brightness?
		float		val = (inputPixelColor.r + inputPixelColor.g + inputPixelColor.b) / 3.0;
		
		//	how much do we shift by based on that brightness?
		if (val < 0.25)	{
			val = shiftLow;
		}
		else if (val < 0.5)	{
			val = mix(shiftLow, shiftMid, (val-0.25) * 4.0);
		}
		else if (val < 0.75)	{
			val = mix(shiftMid, shiftHigh, (val-0.5) * 4.0);
		}
		else	{
			val = shiftHigh;
		}
		
		inputPixelColor.rgb = rgb2hsv(inputPixelColor.rgb);
		inputPixelColor.r = fract(inputPixelColor.r + val);
		inputPixelColor.rgb = hsv2rgb(inputPixelColor.rgb);
		
	}
	
	gl_FragColor = inputPixelColor;
}
