/*{
	"CREDIT": "by zoidberg",
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
			"NAME": "slideAmt",
			"LABEL": "slide amount",
			"TYPE": "color",
			"DEFAULT": [
				0.0,
				0.0,
				0.0,
				0.0
			]
		},
		{
			"NAME": "reflection",
			"TYPE": "bool",
			"DEFAULT": 0.0
		}
	]
}*/

void main() {
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	if (reflection == true)	{
		vec4		outPixel;
		outPixel.rgb = srcPixel.rgb - slideAmt.rgb;
		outPixel.a = srcPixel.a + slideAmt.a;	//	alpha behaves the same in both modes (just easier to work with)
		gl_FragColor.x = (outPixel.x<0.0) ? outPixel.x+1.0 : outPixel.x;
		gl_FragColor.y = (outPixel.y<0.0) ? outPixel.y+1.0 : outPixel.y;
		gl_FragColor.z = (outPixel.z<0.0) ? outPixel.z+1.0 : outPixel.z;
		//gl_FragColor.a = (outPixel.a<0.0) ? outPixel.a+1.0 : outPixel.a;
		gl_FragColor.a = (outPixel.a>1.0) ? outPixel.a-1.0 : outPixel.a;
	}
	else	{
		vec4		outPixel = srcPixel+slideAmt;
		gl_FragColor.x = (outPixel.x>1.0) ? outPixel.x-1.0 : outPixel.x;
		gl_FragColor.y = (outPixel.y>1.0) ? outPixel.y-1.0 : outPixel.y;
		gl_FragColor.z = (outPixel.z>1.0) ? outPixel.z-1.0 : outPixel.z;
		gl_FragColor.a = (outPixel.a>1.0) ? outPixel.a-1.0 : outPixel.a;
	}
}
