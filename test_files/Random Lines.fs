/*{
    "CATEGORIES": [
        "Geometry"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "DEFAULT": 15,
            "LABEL": "Line Count",
            "MAX": 60,
            "MIN": 1,
            "NAME": "lineCount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.025,
            "LABEL": "Max Line Width",
            "MAX": 0.25,
            "MIN": 0,
            "NAME": "lineWidth",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "LABEL": "Random Seed",
            "MAX": 1,
            "MIN": 0.01,
            "NAME": "randomSeed",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Wobble Amount",
            "MAX": 0.1,
            "MIN": 0,
            "NAME": "wobbleAmount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.2,
            "LABEL": "Hue Range",
            "MAX": 1,
            "MIN": 0,
            "NAME": "hueRange",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Saturation",
            "MAX": 1,
            "MIN": 0,
            "NAME": "colorSaturation",
            "TYPE": "float"
        },
        {
            "DEFAULT": true,
            "LABEL": "Randomize Brightness",
            "NAME": "randomizeBrightness",
            "TYPE": "bool"
        },
        {
            "DEFAULT": false,
            "LABEL": "Randomize Width",
            "NAME": "randomizeWidth",
            "TYPE": "bool"
        },
        {
            "DEFAULT": false,
            "LABEL": "Randomize Points",
            "NAME": "randomizeAllPoints",
            "TYPE": "bool"
        },
        {
            "DEFAULT": false,
            "LABEL": "Randomize Alpha",
            "NAME": "randomizeAlpha",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2"
}
*/

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

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

bool is_point_above_line(vec2 pt, float slope, float intercept)	{
	bool returnMe = false;
	float y = slope * pt.x + intercept;
	if (y < pt.y)
		returnMe = true;
	return returnMe;
}

//	returns two values – distrance from the line and the percentage of the way on the line
float distance_from_point_to_line(vec2 pt, vec2 l1, vec2 l2){
	float returnMe = 0.0;
	float a = (l2.y - l1.y);
	float b = (l2.x - l1.x);
	float c = 0.0;
	
	//	if b is zero, this is a vertical line!
	//	in which case distance is based on x distance alone
	if (b == 0.0)	{
		float minY = min(l1.y, l2.y);
		float maxY = max(l1.y, l2.y);
		
		//	if we're between the two points distrance is straight up x diff
		if ((pt.y > minY) && (pt.y < maxY))	{
			returnMe = abs(pt.x-l1.x);
		}
		else	{
			//returnMe = min(distance(pt, l1), distance(pt, l2));
			returnMe = -1.0;
		}
	}
	//	if a is zero, this is a horizontal line
	else if (a == 0.0)	{
		float minX = min(l1.x, l2.x);
		float maxX = max(l1.x, l2.x);
		
		//	if we're between the two points distrance is straight up y diff
		if ((pt.x > minX) && (pt.x < maxX))	{
			returnMe = abs(pt.y - l1.y);
		}
		else	{
			//returnMe = min(distance(pt, l1), distance(pt, l2));
			returnMe = -1.0;
		}
	}
	//	if b isn't 0, solve for c now that we know a, b, and either l1 or l2
	else	{		
		//	here's the tricky bit-
		//	if pt is beyond l1 and l2, we should switch to distance from those points
		//	in order to determine this we need to use the perpendicular lines to the segment l1|l2 that pass through l1 & l2
		//	the slope of the perp line will be -1.0 / slope of original line
		float m = a / b;
		float perpm = -b / a;
		vec2 left_line_pt = l1;
		vec2 right_line_pt = l2;
		if (l1.x > l2.x)	{
			left_line_pt = l2;
			right_line_pt = l1;
		}
		
		float perp_intercept1 = left_line_pt.y - perpm * left_line_pt.x;
		float perp_intercept2 = right_line_pt.y - perpm * right_line_pt.x;
		
		if (m > 0.0)	{
			/*
			if (is_point_above_line(pt, perpm, perp_intercept1)==false)	{
				//returnMe = distance(pt, left_line_pt);
				returnMe = -1.0;
			}
			else if (is_point_above_line(pt, perpm, perp_intercept2)==true)	{
				//returnMe = distance(pt, right_line_pt);
				returnMe = -1.0;
			}
			else	{
				returnMe = abs(a * pt.x - b * pt.y + l2.x * l1.y - l2.y * l1.x) / sqrt(a*a + b*b);
			}
			*/
			returnMe = abs(a * pt.x - b * pt.y + l2.x * l1.y - l2.y * l1.x) / sqrt(a*a + b*b);
		}
		else	{
			/*
			if (is_point_above_line(pt, perpm, perp_intercept1)==true)	{
				//returnMe = distance(pt, left_line_pt);
				returnMe = -1.0;
			}
			else if (is_point_above_line(pt, perpm, perp_intercept2)==false)	{
				//returnMe = distance(pt, right_line_pt);
				returnMe = -1.0;
			}
			else	{
				returnMe = abs(a * pt.x - b * pt.y + l2.x * l1.y - l2.y * l1.x) / sqrt(a*a + b*b);
			}
			*/
			returnMe = abs(a * pt.x - b * pt.y + l2.x * l1.y - l2.y * l1.x) / sqrt(a*a + b*b);
		}
	}
	 
    return returnMe;
}

void main()	{
	float		maxWidth = lineWidth;
	float		minWidth = 1.0 / min(RENDERSIZE.x, RENDERSIZE.y);
	if (maxWidth < minWidth)
		maxWidth = minWidth;
	
	vec4		result = vec4(0.0);
	vec2		thisPoint = isf_FragNormCoord;
	vec3		colorHSL;
	vec2		pt1, pt2;
	float		baseHue = rand(vec2(floor(lineCount), 1.0));
	
	colorHSL.x = baseHue;
	colorHSL.y = colorSaturation;
	colorHSL.z = 1.0;
	if (randomizeBrightness)	{
		colorHSL.z = rand(vec2(floor(lineCount)+randomSeed * 3.72, randomSeed + lineCount * 0.649));
	}
	
	vec2 wobbleVector = vec2(0.0);
	
	pt1 = vec2(rand(vec2(floor(lineCount)+randomSeed*1.123,randomSeed*1.321)),rand(vec2(randomSeed*2.123,randomSeed*3.325)));
	pt2 = vec2(rand(vec2(floor(lineCount)+randomSeed*0.317,randomSeed*2.591)),rand(vec2(randomSeed*1.833,randomSeed*4.916)));
	
	if (wobbleAmount > 0.0)	{
		wobbleVector = wobbleAmount * vec2(rand(vec2(TIME*1.123,TIME*3.239)),rand(vec2(TIME*3.321,TIME*2.131))) - vec2(wobbleAmount / 2.0);
		pt1 = pt1 + wobbleVector;
		
		wobbleVector = wobbleAmount * vec2(rand(vec2(TIME*6.423,TIME*1.833)),rand(vec2(TIME*2.436,TIME*7.532))) - vec2(wobbleAmount / 2.0);
		pt2 = pt2 + wobbleVector;
	}
	
	float		randomWidth = maxWidth;
	
	if (randomizeWidth)	{
		randomWidth = clamp(maxWidth * rand(vec2(1.0 + randomSeed * 4.672, randomSeed * lineCount * 2.523)), minWidth, maxWidth);
	}
	
	if (distance_from_point_to_line(thisPoint, pt1, pt2) < randomWidth)	{
		float newAlpha = 1.0;
		
		if (randomizeAlpha)	{
			newAlpha = 0.25 + 0.5 * rand(vec2(1.0 + floor(lineCount)+randomSeed * 1.938, randomSeed * lineCount * 1.541));
		}
		
		result.rgb = hsv2rgb(colorHSL);
		result.a = result.a + newAlpha;
	}
	
	for (float i = 0.0; i < 60.0; ++i)	{
		if (result.a > 0.75)
			break;
		if (i >= lineCount - 1.0)
			break;
		if (randomizeAllPoints)	{
			pt1 = vec2(rand(vec2(i+randomSeed*1.123,floor(lineCount)+randomSeed*1.321)),rand(vec2((1.0+i)*floor(lineCount)+randomSeed*2.123,i+randomSeed*1.325)));
			
			if (wobbleAmount > 0.0)	{
				wobbleVector = wobbleAmount * vec2(rand(vec2(i*floor(lineCount)+TIME*3.123,i*floor(lineCount)+TIME*3.239)),rand(vec2(i*floor(lineCount)+TIME*3.321,i*floor(lineCount)+TIME*2.131))) - vec2(wobbleAmount / 2.0);
				pt1 = pt1 + wobbleVector;
			}
		}
		else	{
			pt1 = pt2;
		}
		pt2 = vec2(rand(vec2(i*floor(lineCount)+randomSeed*3.573,i+randomSeed*6.273)),rand(vec2(i+randomSeed*9.253,i+randomSeed*7.782)));
		
		if (wobbleAmount > 0.0)	{
			wobbleVector = wobbleAmount * vec2(rand(vec2(i*floor(lineCount)+TIME*3.573,i+randomSeed*6.273)),rand(vec2(i+TIME*9.253,i+TIME*7.782))) - vec2(wobbleAmount / 2.0);
			pt2 = pt2 + wobbleVector;
		}
		
		if (randomizeWidth)	{
			randomWidth = clamp(maxWidth * rand(vec2(i + randomSeed * 4.672, 1.673 + i * randomSeed * 2.523)), minWidth, maxWidth);
		}
		
		if (distance_from_point_to_line(thisPoint, pt1, pt2) < randomWidth)	{
			//result = vec4(1.0);
			float newAlpha = 1.0;
			
			if (randomizeAlpha)	{
				newAlpha = 0.25 + 0.25 * rand(vec2(i + floor(lineCount)+randomSeed * 1.938, randomSeed * lineCount * 1.541));
			}
			
			colorHSL.x = mod(baseHue + hueRange * rand(vec2(floor(lineCount)+randomSeed, i)), 1.0);
			if (randomizeBrightness)	{
				colorHSL.z = 0.15 + 0.85 * rand(vec2(i + floor(lineCount)+randomSeed * 2.78, randomSeed + lineCount * 0.249));
			}
			result.rgb = result.rgb + hsv2rgb(colorHSL) * newAlpha;
			result.a = result.a + newAlpha;
		}
	}
	
	gl_FragColor = result;
}
