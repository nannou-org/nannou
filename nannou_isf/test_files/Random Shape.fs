/*{
    "CATEGORIES": [
        "Geometry"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "DEFAULT": 15,
            "LABEL": "Point Count",
            "MAX": 90,
            "MIN": 3,
            "NAME": "pointCount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.125,
            "LABEL": "Random Seed",
            "MAX": 1,
            "MIN": 0.01,
            "NAME": "randomSeed",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Wobble Amount",
            "MAX": 0.25,
            "MIN": 0,
            "NAME": "wobbleAmount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.75,
            "LABEL": "Zoom Start",
            "MAX": 4,
            "MIN": 0.001,
            "NAME": "zoomStart",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Zoom End",
            "MAX": 4,
            "MIN": 0.001,
            "NAME": "zoomEnd",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Winding Start",
            "MAX": 4,
            "MIN": -4,
            "NAME": "rotationStart",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Winding End",
            "MAX": 4,
            "MIN": -4,
            "NAME": "rotationEnd",
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
            "DEFAULT": 0.2,
            "LABEL": "Hue Base",
            "MAX": 1,
            "MIN": 0,
            "NAME": "hueBase",
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
            "DEFAULT": [
                0.5,
                0.5
            ],
            "MAX": [
                1,
                1
            ],
            "MIN": [
                0,
                0
            ],
            "NAME": "offsetEnd",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": true,
            "LABEL": "Randomize Brightness",
            "NAME": "randomizeBrightness",
            "TYPE": "bool"
        },
        {
            "DEFAULT": false,
            "LABEL": "Randomize Alpha",
            "NAME": "randomizeAlpha",
            "TYPE": "bool"
        },
        {
            "DEFAULT": false,
            "LABEL": "Randomize Points",
            "NAME": "randomizeAllPoints",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2"
}
*/



const float pi = 3.14159265359;



vec2 rotatePoint(vec2 pt, float angle, vec2 center)
{
	vec2 returnMe;
	float s = sin(angle * pi);
	float c = cos(angle * pi);

	returnMe = pt;

	// translate point back to origin:
	returnMe.x -= center.x;
	returnMe.y -= center.y;

	// rotate point
	float xnew = returnMe.x * c - returnMe.y * s;
	float ynew = returnMe.x * s + returnMe.y * c;

	// translate point back:
	returnMe.x = xnew + center.x;
	returnMe.y = ynew + center.y;
	return returnMe;
}


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


void main() {
	vec4		result = vec4(0.0);
	vec2		thisPoint = isf_FragNormCoord;
	vec3		colorHSL;
	vec2		pt1, pt2, pt3;
	vec2		offsetIncrement = (4.0 * (offsetEnd - vec2(0.5)) / (pointCount - 2.0));
	float		rotationIncrement = (rotationEnd - rotationStart) / pointCount;
	float		zoomIncrement = (zoomEnd - zoomStart) / pointCount;
	
	thisPoint = rotatePoint(thisPoint, rotationStart, RENDERSIZE/2.0);
	thisPoint = thisPoint / RENDERSIZE;
	thisPoint = (thisPoint - vec2(0.5)) / zoomStart + vec2(0.5);
	
	colorHSL.x = hueBase;
	colorHSL.y = colorSaturation;
	colorHSL.z = 1.0;
	if (randomizeBrightness)	{
		colorHSL.z = rand(vec2(floor(pointCount)+randomSeed * 3.72, randomSeed + pointCount * 0.649));
	}
	
	vec2 wobbleVector = vec2(0.0);
	
	pt1 = vec2(rand(vec2(floor(pointCount)+randomSeed*1.123,randomSeed*1.321)),rand(vec2(randomSeed*2.123,randomSeed*3.325)));
	pt2 = vec2(rand(vec2(floor(pointCount)+randomSeed*5.317,randomSeed*2.591)),rand(vec2(randomSeed*1.833,randomSeed*4.916)));
	pt3 = vec2(rand(vec2(floor(pointCount)+randomSeed*3.573,randomSeed*6.273)),rand(vec2(randomSeed*9.253,randomSeed*7.782)));
	
	if (wobbleAmount > 0.0)	{
		wobbleVector = wobbleAmount * vec2(rand(vec2(TIME*1.123,TIME*3.239)),rand(vec2(TIME*3.321,TIME*2.131))) - vec2(wobbleAmount / 2.0);
		pt1 = pt1 + wobbleVector;
		
		wobbleVector = wobbleAmount * vec2(rand(vec2(TIME*6.423,TIME*1.833)),rand(vec2(TIME*2.436,TIME*7.532))) - vec2(wobbleAmount / 2.0);
		pt2 = pt2 + wobbleVector;
		
		wobbleVector = wobbleAmount * vec2(rand(vec2(TIME*3.951,TIME*3.538)),rand(vec2(TIME*8.513,TIME*6.335))) - vec2(wobbleAmount / 2.0);
		pt3 = pt3 + wobbleVector;
	}
	
	if (PointInTriangle(thisPoint,pt1,pt2,pt3))	{
		float newAlpha = 1.0;
		
		if (randomizeAlpha)	{
			newAlpha = 0.5 + 0.5 * rand(vec2(1.0 + floor(pointCount)+randomSeed * 1.938, randomSeed * pointCount * 1.541));
		}
		
		result.rgb = hsv2rgb(colorHSL);
		result.a = result.a + newAlpha;
	}
	
	for (float i = 0.0; i < 90.0; ++i)	{
		if (result.a > 0.75)
			break;
		if (i > pointCount - 3.0)
			break;
		if (randomizeAllPoints)	{
			pt1 = vec2(rand(vec2(i+randomSeed*1.123,i*floor(pointCount)+randomSeed*1.321)),rand(vec2(i*floor(pointCount)+randomSeed*2.123,i+randomSeed*1.325)));
			pt2 = vec2(rand(vec2(i*floor(pointCount)+randomSeed*5.317,randomSeed*2.591)),rand(vec2(i+randomSeed*1.833,i*floor(pointCount)+randomSeed*4.916)));
			
			if (wobbleAmount > 0.0)	{
				wobbleVector = wobbleAmount * vec2(rand(vec2(i*floor(pointCount)+TIME*3.123,i*floor(pointCount)+TIME*3.239)),rand(vec2(i*floor(pointCount)+TIME*3.321,i*floor(pointCount)+TIME*2.131))) - vec2(wobbleAmount / 2.0);
				pt1 = pt1 + wobbleVector;
			
				wobbleVector = wobbleAmount * vec2(rand(vec2(i*floor(pointCount)+TIME*6.423,i*floor(pointCount)+TIME*1.833)),rand(vec2(i*floor(pointCount)+TIME*2.436,i*floor(pointCount)+TIME*7.532))) - vec2(wobbleAmount / 2.0);
				pt2 = pt2 + wobbleVector;
			}
		}
		else	{
			pt1 = pt2;
			pt2 = pt3;
		}
		pt3 = vec2(rand(vec2(i*floor(pointCount)+randomSeed*3.573,i+randomSeed*6.273)),rand(vec2(i+randomSeed*9.253,i+randomSeed*7.782)));
		pt3 = (pt3 - vec2(0.5)) * (zoomStart + zoomIncrement * i) + vec2(0.5);
		pt3 = rotatePoint(pt3, rotationStart + rotationIncrement * i, vec2(0.5));
		pt3 = pt3 + offsetIncrement * i;
		
		if (wobbleAmount > 0.0)	{
			wobbleVector = wobbleAmount * vec2(rand(vec2(i*floor(pointCount)+TIME*3.573,i+randomSeed*6.273)),rand(vec2(i+TIME*9.253,i+TIME*7.782))) - vec2(wobbleAmount / 2.0);
			pt3 = pt3 + wobbleVector;
		}
		
		if (PointInTriangle(thisPoint,pt1,pt2,pt3))	{
			//result = vec4(1.0);
			float newAlpha = 1.0;
			
			if (randomizeAlpha)	{
				newAlpha = 0.1 + 0.25 * rand(vec2(i + floor(pointCount)+randomSeed * 1.938, randomSeed * pointCount * 1.541));
			}
			
			colorHSL.x = mod(hueBase + hueRange * rand(vec2(floor(pointCount)+randomSeed, i)), 1.0);
			if (randomizeBrightness)	{
				colorHSL.z = 0.25 + 0.85 * rand(vec2(i + floor(pointCount)+randomSeed * 2.78, randomSeed + pointCount * 0.249));
			}
			result.rgb = result.rgb + hsv2rgb(colorHSL) * newAlpha;
			result.a = result.a + newAlpha;
		}
	}
	
	gl_FragColor = result;
}