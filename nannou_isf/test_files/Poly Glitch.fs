/*{
	"CREDIT": "by VIDVOX",
	"CATEGORIES": [
		"Stylize", "Glitch"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "randomSeed",
			"LABEL": "Random Seed",
			"TYPE": "float",
			"MIN": 0.01,
			"MAX": 1.0,
			"DEFAULT": 0.239
		},
		{
			"NAME": "sizeSpread",
			"LABEL": "Poly Size",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 0.25,
			"DEFAULT": 0.025
		},
		{
			"NAME": "sizeGain",
			"LABEL": "Size Gain",
			"TYPE": "float",
			"MIN": 1.0,
			"MAX": 2.0,
			"DEFAULT": 1.0
		},
		{
			"NAME": "sampleMode",
			"LABEL": "Sample Mode",
			"VALUES": [
				0,
				1
			],
			"LABELS": [
				"Single",
				"Double"
			],
			"DEFAULT": 0,
			"TYPE": "long"
		}
	]
}*/


const float MaxPointCount = 5.0;
float divisions = 1.0 / sizeSpread;



float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

float sign(vec2 p1, vec2 p2, vec2 p3)	{
	return (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y);
}

bool PointInTriangle(vec2 pt, vec2 v1, vec2 v2, vec2 v3)	{
	bool b1, b2, b3;

	b1 = sign(pt, v1, v2) < 0.0;
	b2 = sign(pt, v2, v3) < 0.0;
	b3 = sign(pt, v3, v1) < 0.0;

	return ((b1 == b2) && (b2 == b3));
}

vec2 AvgPointForTriangle(vec2 p1, vec2 p2, vec2 p3)	{
	return (p1 + p2 + p3) / 3.0;	
}

float AreaOfTriangle(vec2 p1, vec2 p2, vec2 p3)	{
	return abs(p1.x*(p2.y-p3.y)+p2.x*(p3.y-p1.y)+p3.x*(p1.y-p2.y))/2.0;	
}

vec4 ColorForPtIndex(vec2 index1, vec2 index2, float sizeAmount, float theSeed, vec2 pt)	{
	vec4	returnMe = vec4(0.0);
	vec2	pt1, pt2, pt3;
	vec2	offset1 = (index1 + divisions / 2.0) * sizeAmount + sizeAmount / 2.0;
	vec2	offset2 = (index2 + divisions / 2.0) * sizeAmount + sizeAmount / 2.0;
	float	adjustedSizeAmount = sizeAmount * sizeGain;
	vec2	inPoint = pt + vec2(0.5);
	
	
	pt1 = vec2(rand(vec2(theSeed+index1.x,2.187*theSeed+index1.x)),rand(vec2(theSeed+index1.y,0.823*theSeed+index1.y)));
	pt2 = pt1 + vec2(rand(vec2(theSeed+index1.x,5.326*theSeed+index1.x)),rand(vec2(theSeed+index1.y,1.421*theSeed+index1.y)));
	pt3 = pt1 - vec2(rand(vec2(theSeed+index2.x,1.935*theSeed+index2.x)),rand(vec2(theSeed+index2.y,3.177*theSeed+index2.y)));
	
	float	area = AreaOfTriangle(pt1,pt2,pt3);
	
	if (area < 0.5)	{
		pt1 = 1.0 - pt1;
		adjustedSizeAmount = adjustedSizeAmount * (2.5-area);
	}
	else if (area > 0.5)	{
		//pt3 = 1.0-pt3;
	}
	
	pt1 = clamp(pt1 * adjustedSizeAmount + offset1 - adjustedSizeAmount / 2.0,0.0,1.0);
	pt2 = clamp(pt2 * adjustedSizeAmount + offset1 - adjustedSizeAmount / 2.0,0.0,1.0);
	pt3 = clamp(pt3 * adjustedSizeAmount + offset2 - adjustedSizeAmount / 2.0,0.0,1.0);
	
	if(PointInTriangle(inPoint,pt1,pt2,pt3))	{
		vec2	avgPt = AvgPointForTriangle(pt1,pt2,pt3);
		returnMe = IMG_NORM_PIXEL(inputImage,avgPt);
		if (sampleMode==1)	{
			returnMe.a = 0.5;
			returnMe.rgb = returnMe.rgb * returnMe.a;
		}
		else	{
			returnMe.a = 1.0;
		}
	}
	
	return returnMe;
}



void main() {	
	vec4		result = vec4(0.0);
	vec2		thisPoint = vv_FragNormCoord - vec2(0.5);
	float		inSpread = sizeSpread;
	vec2		ptIndex = floor(thisPoint / inSpread);
	vec4		original = IMG_THIS_NORM_PIXEL(inputImage);
	float		pointCount = MaxPointCount + float(sampleMode);
	
	if (sizeSpread > 0.0)	{		
		inSpread = sizeSpread * 1.0;
		for (float i=0.0;i<pointCount;++i)	{
			if (result.a>0.667)
				break;
			float	tmpSpread = inSpread;
			if (result.a<=0.75)
				result += ColorForPtIndex(ptIndex,ptIndex+vec2(1.0,0.0),tmpSpread,i+randomSeed,thisPoint);
			if (result.a<=0.75)
				result += ColorForPtIndex(ptIndex,ptIndex+vec2(-1.0,0.0),tmpSpread,i+randomSeed,thisPoint);
			if (result.a<=0.75)
				result += ColorForPtIndex(ptIndex,ptIndex+vec2(0.0,1.0),tmpSpread,i+randomSeed,thisPoint);
			if (result.a<=0.75)
				result += ColorForPtIndex(ptIndex,ptIndex+vec2(0.0,-1.0),tmpSpread,i+randomSeed,thisPoint);	
			
			if (result.a<=0.75)
				result += ColorForPtIndex(ptIndex+vec2(-1.0,0.0),ptIndex,tmpSpread,i+randomSeed,thisPoint);
			if (result.a<=0.75)
				result += ColorForPtIndex(ptIndex+vec2(0.0,1.0),ptIndex,tmpSpread,i+randomSeed,thisPoint);
			if (result.a<=0.75)
				result += ColorForPtIndex(ptIndex+vec2(0.0,-1.0),ptIndex,tmpSpread,i+randomSeed,thisPoint);
			if (result.a<=0.75)
				result += ColorForPtIndex(ptIndex+vec2(1.0,0.0),ptIndex,tmpSpread,i+randomSeed,thisPoint);
		}
		if (result.a<1.0)
			result += (2.0 - result.a) * ColorForPtIndex(ptIndex,ptIndex,inSpread,randomSeed,thisPoint);
		if (sampleMode==0)
			result.a = original.a;
	}
	else	{
		result = original;
	}
	gl_FragColor = result;
}