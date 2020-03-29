#if __VERSION__ <= 120
varying vec2		texOffsets[3];
#else
out vec2		texOffsets[3];
#endif

const float radius = 10.0;

void main(void)	{
	//	load the main shader stuff
	isf_vertShaderInit();
	
	if (PASSINDEX==0 || PASSINDEX==2 || PASSINDEX==4 || PASSINDEX==6 || PASSINDEX==8)	{
		float		pixelWidth = 1.0/RENDERSIZE[0]*radius;
		if (PASSINDEX >= 2)
			pixelWidth *= .7;
		else if (PASSINDEX >= 6)
			pixelWidth *= 1.0;
		texOffsets[0] = isf_FragNormCoord;
		texOffsets[1] = clamp(vec2(isf_FragNormCoord[0]-pixelWidth, isf_FragNormCoord[1]),0.0,1.0);
		texOffsets[2] = clamp(vec2(isf_FragNormCoord[0]+pixelWidth, isf_FragNormCoord[1]),0.0,1.0);
	}
	else if (PASSINDEX==1 || PASSINDEX==3 || PASSINDEX==5 || PASSINDEX==7 || PASSINDEX==9)	{
		float		pixelHeight = 1.0/RENDERSIZE[1]*radius;
		if (PASSINDEX >= 3)
			pixelHeight *= .7;
		else if (PASSINDEX >= 6)
			pixelHeight *= 1.0;
		texOffsets[0] = isf_FragNormCoord;
		texOffsets[1] = clamp(vec2(isf_FragNormCoord[0], isf_FragNormCoord[1]-pixelHeight),0.0,1.0);
		texOffsets[2] = clamp(vec2(isf_FragNormCoord[0], isf_FragNormCoord[1]+pixelHeight),0.0,1.0);
	}
}
