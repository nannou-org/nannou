#if __VERSION__ <= 120
varying vec2		texOffsets[5];

varying vec2 left_coord;
varying vec2 right_coord;
varying vec2 above_coord;
varying vec2 below_coord;

varying vec2 lefta_coord;
varying vec2 righta_coord;
varying vec2 leftb_coord;
varying vec2 rightb_coord;
#else
out vec2		texOffsets[5];

out vec2 left_coord;
out vec2 right_coord;
out vec2 above_coord;
out vec2 below_coord;

out vec2 lefta_coord;
out vec2 righta_coord;
out vec2 leftb_coord;
out vec2 rightb_coord;
#endif

void main(void)	{
	//	load the main shader stuff
	isf_vertShaderInit();
	
	
	int			blurLevel = int(floor(blurAmount/6.0));
	float		blurLevelModulus = mod(blurAmount, 6.0);
	float		blurRadius = 0.0;
	float		blurRadiusInPixels = 0.0;
	//	first three passes are just drawing the texture- do nothing
	
	//	the next four passes do gaussion blurs on the various levels
	if (PASSINDEX==3 || PASSINDEX==5)	{
		float		pixelWidth = 1.0/RENDERSIZE.x;
		//	pass 3 is sampling 1/12, but writing to 1/4
		if (PASSINDEX==3)	{
			if (blurLevel==1)
				blurRadius = blurLevelModulus/1.5;
			else if (blurLevel>1)
				blurRadius = 4.0;
		}
		//	pass 5 is sampling 1/4 and writing to full-size
		else if (PASSINDEX==5)	{
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
	else if (PASSINDEX==4 || PASSINDEX==6)	{
		float		pixelHeight = 1.0/RENDERSIZE.y;
		//	pass 4 is sampling 1/12, but writing to 1/4
		if (PASSINDEX==4)	{
			if (blurLevel==1)
				blurRadius = blurLevelModulus/1.5;
			else if (blurLevel>1)
				blurRadius = 4.0;
		}
		//	pass 6 is sampling 1/4 and writing to full-size
		else if (PASSINDEX==6)	{
			if (blurLevel==0)
				blurRadius = blurLevelModulus;
			else if (blurLevel>0)
				blurRadius = 6.0;
			
			vec2 texc = vec2(isf_FragNormCoord[0],isf_FragNormCoord[1]);
			vec2 d = 1.0/RENDERSIZE;
		
			left_coord = clamp(vec2(texc.xy + vec2(-d.x , 0)),0.0,1.0);
			right_coord = clamp(vec2(texc.xy + vec2(d.x , 0)),0.0,1.0);
			above_coord = clamp(vec2(texc.xy + vec2(0,d.y)),0.0,1.0);
			below_coord = clamp(vec2(texc.xy + vec2(0,-d.y)),0.0,1.0);
	
			lefta_coord = clamp(vec2(texc.xy + vec2(-d.x , d.x)),0.0,1.0);
			righta_coord = clamp(vec2(texc.xy + vec2(d.x , d.x)),0.0,1.0);
			leftb_coord = clamp(vec2(texc.xy + vec2(-d.x , -d.x)),0.0,1.0);
			rightb_coord = clamp(vec2(texc.xy + vec2(d.x , -d.x)),0.0,1.0);
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
