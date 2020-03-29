/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Stylize", "Distortion Effect"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "width",
			"LABEL": "Width",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "angle",
			"LABEL": "Angle",
			"TYPE": "float",
			"MIN": -0.5,
			"MAX": 0.5,
			"DEFAULT": 0.125
		}
	]
}*/


const float pi = 3.14159265359;


void main() {
	vec2 loc = isf_FragNormCoord * RENDERSIZE;

	//	for starters let's just do y = x * tan(angle) + b
	
	float b = 0.0;
	float tanVal = 0.0;
	vec2 p1 = vec2(0.0);
	vec2 p2 = vec2(1.0);
	
	if (abs(angle) != 0.5)	{
		tanVal = tan(pi * angle);
		b = (loc.y - loc.x * tanVal)/RENDERSIZE.y;
		
		if (width > 0.0)	{
			float w = width * (1.0+abs(tanVal));
			b = w * floor(b/w);
		}
		b = b * RENDERSIZE.y;
		
		//	if p1 is offscreen, adjust to an onscreen point
		p1 = vec2(0.0, b);
		
		//	in this case instead of using where it hits the left edge, use where it hits the bottom, solving x for y = 0.0 (x = (y - b) / tanVal)
		if (p1.y < 0.0)	{
			p1 = vec2(0.0-b / tanVal, 0.0);	
		}
		//	in this case instead of using where it hits the left edge, use where it hits the top, solving x for y = 1.0 (x = (y - b) / tanVal)
		else if (p1.y > RENDERSIZE.y)	{
			p1 = vec2((RENDERSIZE.x - b) / tanVal, RENDERSIZE.y);
		}
	
		//	get the right side edge
		p2 = vec2(RENDERSIZE.x, RENDERSIZE.x * tanVal + b);
		
		//	if p2 is offscreen, adjust to an onscreen point
		//	in this case instead of using where it hits the right edge, use where it hits the bottom, solving x for y = 0.0 (x = (y - b) / tanVal)
		if (p2.y < 0.0)	{
			p2 = vec2(- b / tanVal, 0.0);
		}
		//	in this case instead of using where it hits the right edge, use where it hits the top, solving x for y = 1.0 (x = (y - b) / tanVal)
		else if (p2.y > RENDERSIZE.y)	{
			p2 = vec2((RENDERSIZE.y - b) / tanVal, RENDERSIZE.y);
		}
		
	}
	//	vertical lines! set p1 & p2 to fixed x with y = 0.0 and 1.0
	else	{
		if (angle > 0.0)	{
			p1 = vec2(loc.x, 0.0);
			p2 = vec2(loc.x, RENDERSIZE.y);
		}
		else	{
			p2 = vec2(loc.x, 0.0);
			p1 = vec2(loc.x, RENDERSIZE.y);	
		}
		if (width > 0.0)	{
			p1.x = RENDERSIZE.x * width * floor((loc.x/RENDERSIZE.x) / width);
			p2.x = p1.x;
		}
	}
	
	//	now average 5 points on the line, p1, p2, their midpoint, and the mid points of those
	//	midpoint of the line
	vec2 mid = (p1 + p2) / 2.0;
	
	vec4 returnMe = (IMG_PIXEL(inputImage, p1) + IMG_PIXEL(inputImage, (p1 + mid) / 2.0) + IMG_PIXEL(inputImage, mid) + IMG_PIXEL(inputImage, (p2 + mid) / 2.0) + IMG_PIXEL(inputImage, p2)) / 5.0;
	//vec4 returnMe = IMG_PIXEL(inputImage, p2);

	gl_FragColor = returnMe;
}



