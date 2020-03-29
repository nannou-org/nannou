/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Geometry Adjustment"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "horizontal",
			"TYPE": "bool",
			"DEFAULT": 1.0
		},
		{
			"NAME": "vertical",
			"TYPE": "bool",
			"DEFAULT": 0.0
		}
	]
}*/

void main() {
	//	isf_FragNormCoord[0] and isf_FragNormCoord[1] are my normalized x/y coordinates
	//	if we're not doing a flip in either direction we can just pass thru
	vec2		normSrcCoord;

	normSrcCoord.x = isf_FragNormCoord[0];
	normSrcCoord.y = isf_FragNormCoord[1];

	if ((normSrcCoord.x > 0.5)&&(horizontal))	{
		normSrcCoord.x = (1.0-normSrcCoord.x);
	}
	if ((normSrcCoord.y > 0.5)&&(vertical))	{
		normSrcCoord.y = (1.0-normSrcCoord.y);
	}
	
	gl_FragColor = IMG_NORM_PIXEL(inputImage, normSrcCoord);
}