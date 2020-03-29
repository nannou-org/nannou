#if __VERSION__ <= 120
varying vec2		texOffsets[5];
#else
out vec2		texOffsets[5];
#endif

void main(void)	{
	//	load the main shader stuff
	isf_vertShaderInit();
	
	
	int			blurLevel = int(floor(blurAmount/6.0));
	float		blurLevelModulus = mod(blurAmount, 6.0);
	float		blurRadius = 0.0;
	float		blurRadiusInPixels = 0.0;
	//	first three passes are just drawing the texture- do nothing
	
	//	the next six passes do gaussion blurs on the various levels
	if (PASSINDEX==3 || PASSINDEX==5 || PASSINDEX==7 || PASSINDEX==9)	{
		float		pixelWidth = 1.0/RENDERSIZE.x;
		//	pass 3 is eighth-size (blurLevel 3)
		if (PASSINDEX==3)	{
			if (blurLevel==3)
				blurRadius = blurLevelModulus/2.0;
			else if (blurLevel>3)
				blurRadius = 3.0;
		}
		//	pass 5 is quarter-size (blurLevel 2)
		else if (PASSINDEX==5)	{
			if (blurLevel==2)
				blurRadius = blurLevelModulus/1.5;
			else if (blurLevel>2)
				blurRadius = 4.0;
		}
		//	pass 7 is half-size (blurLevel 1)
		else if (PASSINDEX==7)	{
			if (blurLevel==1)
				blurRadius = blurLevelModulus;
			else if (blurLevel>1)
				blurRadius = 6.0;
		}
		//	pass 9 is normal-size (blurLevel 0)
		else if (PASSINDEX==9)	{
			if (blurLevel==0)
				blurRadius = blurLevelModulus;
			else if (blurLevel>0)
				blurRadius = 6.0;
		}
		blurRadiusInPixels = pixelWidth * blurRadius;
		texOffsets[0] = isf_FragNormCoord;
		texOffsets[1] = (blurRadius==0.0) ? isf_FragNormCoord : clamp(vec2(isf_FragNormCoord[0]-blurRadiusInPixels, isf_FragNormCoord[1]),0.0,1.0);
		texOffsets[2] = (blurRadius==0.0) ? isf_FragNormCoord : clamp(vec2(isf_FragNormCoord[0]+blurRadiusInPixels, isf_FragNormCoord[1]),0.0,1.0);
		if (PASSINDEX==3)	{
			texOffsets[3] = (blurRadius==0.0) ? isf_FragNormCoord : clamp(vec2(isf_FragNormCoord[0]-(2.0*blurRadiusInPixels), isf_FragNormCoord[1]),0.0,1.0);
			texOffsets[4] = (blurRadius==0.0) ? isf_FragNormCoord : clamp(vec2(isf_FragNormCoord[0]+(2.0*blurRadiusInPixels), isf_FragNormCoord[1]),0.0,1.0);
		}
	}
	else if (PASSINDEX==4 || PASSINDEX==6 || PASSINDEX==8 || PASSINDEX==10)	{
		float		pixelHeight = 1.0/RENDERSIZE.y;
		//	pass 4 is eighth-size (blurLevel 3)
		if (PASSINDEX==4)	{
			if (blurLevel==3)
				blurRadius = blurLevelModulus/2.0;
			else if (blurLevel>3)
				blurRadius = 3.0;
		}
		//	pass 6 is quarter-size (blurLevel 2)
		else if (PASSINDEX==6)	{
			if (blurLevel==2)
				blurRadius = blurLevelModulus/1.5;
			else if (blurLevel>2)
				blurRadius = 4.0;
		}
		//	pass 8 is half-size (blurLevel 1)
		else if (PASSINDEX==8)	{
			if (blurLevel==1)
				blurRadius = blurLevelModulus;
			else if (blurLevel>1)
				blurRadius = 6.0;
		}
		//	pass 10 is normal-size (blurLevel 0)
		else if (PASSINDEX==10)	{
			if (blurLevel==0)
				blurRadius = blurLevelModulus;
			else if (blurLevel>0)
				blurRadius = 6.0;
		}
		blurRadiusInPixels = pixelHeight * blurRadius;
		texOffsets[0] = isf_FragNormCoord;
		texOffsets[1] = (blurRadius==0.0) ? isf_FragNormCoord : clamp(vec2(isf_FragNormCoord[0], isf_FragNormCoord[1]-blurRadiusInPixels),0.0,1.0);
		texOffsets[2] = (blurRadius==0.0) ? isf_FragNormCoord : clamp(vec2(isf_FragNormCoord[0], isf_FragNormCoord[1]+blurRadiusInPixels),0.0,1.0);
		if (PASSINDEX==4)	{
			texOffsets[3] = (blurRadius==0.0) ? isf_FragNormCoord : clamp(vec2(isf_FragNormCoord[0], isf_FragNormCoord[1]-(2.0*blurRadiusInPixels)),0.0,1.0);
			texOffsets[4] = (blurRadius==0.0) ? isf_FragNormCoord : clamp(vec2(isf_FragNormCoord[0], isf_FragNormCoord[1]+(2.0*blurRadiusInPixels)),0.0,1.0);
		}
	}
	
}
