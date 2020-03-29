#if __VERSION__ <= 120
varying vec2 translated_coord;
#else
out vec2 translated_coord;
#endif

const float pi = 3.14159265359;

void main()	{
	isf_vertShaderInit();

	vec2 loc = RENDERSIZE * vec2(vv_FragNormCoord[0],vv_FragNormCoord[1]);
	float r = distance(RENDERSIZE/2.0, loc);
	float a = atan ((loc.y-RENDERSIZE.y/2.0),(loc.x-RENDERSIZE.x/2.0));

	loc.x = r * cos(a + 2.0 * pi * angle) + 0.5;
	loc.y = r * sin(a + 2.0 * pi * angle) + 0.5;
	
	translated_coord = loc / RENDERSIZE + vec2(0.5);

}