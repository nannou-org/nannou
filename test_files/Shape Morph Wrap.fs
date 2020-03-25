/*{
    "CATEGORIES": [
        "Distortion Effect"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "Wraps an image into a shape that is created by morphing two primitive shapes together",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "mixPoint",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABELS": [
                "Circle",
                "Triangle",
                "Rect",
                "Pentagram",
                "Hexagon",
                "Star1",
                "Star2",
                "Heart",
                "Rays"
            ],
            "NAME": "shape1",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2,
                3,
                4,
                5,
                6,
                7,
                8
            ]
        },
        {
            "DEFAULT": 1,
            "LABELS": [
                "Circle",
                "Triangle",
                "Rect",
                "Pentagram",
                "Hexagon",
                "Star1",
                "Star2",
                "Heart",
                "Rays"
            ],
            "NAME": "shape2",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2,
                3,
                4,
                5,
                6,
                7,
                8
            ]
        },
        {
            "DEFAULT": 0,
            "MAX": 2,
            "MIN": 0,
            "NAME": "shapeWobble",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "preRotateAngle",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.75,
            "MAX": 1,
            "MIN": 0,
            "NAME": "angleShift",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABELS": [
                "None",
                "Extend",
                "Repeat",
                "Reflect"
            ],
            "NAME": "repeatStyle",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2,
                3
            ]
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "repeatDecay",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "resultSize",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 2,
            "MIN": 0,
            "NAME": "resultWidth",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "resultAngle",
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
            "NAME": "resultCenter",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": 1,
            "NAME": "mirrorX",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0,
            "NAME": "mirrorY",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2",
    "VSN": "1"
}
*/


const float pi = 3.1415926535897932384626433832795;
const float tau =  6.2831853071795864769252867665590;

vec2 rotatePoint(vec2 pt, float rot)	{
	vec2 returnMe = pt * RENDERSIZE;

	float r = distance(RENDERSIZE/2.0, returnMe);
	float a = atan ((returnMe.y-RENDERSIZE.y/2.0),(returnMe.x-RENDERSIZE.x/2.0));

	returnMe.x = r * cos(a + 2.0 * pi * rot - pi) + 0.5;
	returnMe.y = r * sin(a + 2.0 * pi * rot - pi) + 0.5;
	
	returnMe = returnMe / RENDERSIZE + vec2(0.5);
	
	return returnMe;
}

vec2 rotatePointNorm(vec2 pt, float rot)	{
	vec2 returnMe = pt;

	float r = distance(vec2(0.50), returnMe);
	float a = atan((returnMe.y-0.5),(returnMe.x-0.5));

	returnMe.x = r * cos(a + 2.0 * pi * rot - pi) + 0.5;
	returnMe.y = r * sin(a + 2.0 * pi * rot - pi) + 0.5;
	
	returnMe = returnMe;
	
	return returnMe;
}
vec2 rotatePointNormPt(vec2 pt, float rot, vec2 rpt)	{
	vec2 returnMe = pt;

	float r = distance(vec2(0.50), returnMe);
	float a = atan((returnMe.y-rpt.y),(returnMe.x-rpt.x));

	returnMe.x = r * cos(a + 2.0 * pi * rot - pi) + rpt.x;
	returnMe.y = r * sin(a + 2.0 * pi * rot - pi) + rpt.y;
	
	returnMe = returnMe;
	
	return returnMe;
}

//	borrowed from pixel spirit deck!
//	https://github.com/patriciogonzalezvivo/PixelSpiritDeck/tree/master/lib

float triSDF(vec2 st) {
    st = (st*2.-1.)*2.;
    return max(abs(st.x) * 0.866025 + st.y * 0.5, -st.y * 0.5);
}
float circleSDF(vec2 st) {
    return length(st-.5)*2.;
}
float polySDF(vec2 st, int V) {
    st = st*2.-1.;
    float a = atan(st.x,st.y)+pi;
    float r = length(st);
    float v = tau/float(V);
    return cos(floor(.5+a/v)*v-a)*r;
}
float pentSDF(vec2 st)	{
	vec2 pt = st;
	pt.y /= 0.89217;
	return polySDF(pt, 5);
}
float hexSDF(vec2 st) {
    st = abs(st*2.-1.);
    return max(abs(st.y), st.x * 0.866025 + st.y * 0.5);
}
float flowerSDF(vec2 st, int N) {
    st = st*2.-1.;
    float r = length(st)*2.;
    float a = atan(st.y,st.x);
    float v = float(N)*.5;
    return 1.-(abs(cos(a*v))*.5+.5)/r;
}
float heartSDF(vec2 st) {
    st -= vec2(.5,.8);
    float r = length(st)*5.5;
    st = normalize(st);
    return r - 
         ((st.y*pow(abs(st.x),0.67))/ 
         (st.y+1.5)-(2.)*st.y+1.26);
}
float starSDF(vec2 st, int V, float s) {
    st = st*4.-2.;
    float a = atan(st.y, st.x)/tau;
    float seg = a * float(V);
    a = ((floor(seg) + 0.5)/float(V) + 
        mix(s,-s,step(.5,fract(seg)))) 
        * tau;
    return abs(dot(vec2(cos(a),sin(a)),
                   st));
}
float raysSDF(vec2 st, int N) {
    st -= .5;
    return fract(atan(st.y,st.x)/tau*float(N));
}

float shapeForType(vec2 st, int shape)	{
	if (shape == 0)
		return circleSDF(st);
	else if (shape == 1)	
		return triSDF(st);
	else if (shape == 2)
		return polySDF(st,4);
	else if (shape == 3)
		return pentSDF(st);
	else if (shape == 4)
		return hexSDF(st);
	else if (shape == 5)
		return starSDF(st,5,0.07);
	else if (shape == 6)
		return starSDF(st,12,0.12);
	else if (shape == 7)
		return heartSDF(st);
	else if (shape == 8)
		return raysSDF(st,6);
}

void main()	{
	vec4		returnMe = vec4(0.0);
	vec2		st = gl_FragCoord.xy/RENDERSIZE;

	st += (0.5 - resultCenter);
	st = rotatePoint(st,resultAngle);
	
    //	size
	st -= 0.5;
	//st += (0.5 - resultCenter);
	st /= max(0.000001,resultSize);
	st.x /= resultWidth;
	st += 0.5;
   
	st = mix(vec2((st.x*RENDERSIZE.x/RENDERSIZE.y)-(RENDERSIZE.x*.5-RENDERSIZE.y*.5)/RENDERSIZE.y,st.y), 
				vec2(st.x,st.y*(RENDERSIZE.y/RENDERSIZE.x)-(RENDERSIZE.y*.5-RENDERSIZE.x*.5)/RENDERSIZE.x), 
				step(RENDERSIZE.x,RENDERSIZE.y));
	float		val1 = shapeForType(st,shape1);
	float		val2 = shapeForType(st,shape2);
	//val1 = min(max(val1,0.0),1.0);
	//val2 = min(max(val2,0.0),1.0);
	
	float		val = mix(val1,val2,mixPoint);
	vec2		cnt = vec2(0.5,0.5);
	float		a = (atan(cnt.y-st.y,cnt.x-st.x) + pi) / (tau);
	
	val += (shapeWobble == 0.0) ? 0.0 : shapeWobble * ((sin(TIME+10.0*tau*(a)))/13.0 + (sin(-TIME*2.1+17.0*tau*(a)))/17.0 + (sin(19.0*tau*(a)))/19.0);
	
	float		r = val;
	if (repeatStyle == 1)
		r = max(0.0,min(r,1.0));
	else if (repeatStyle == 2)
		r = mod(r,1.0);
	else if (repeatStyle == 3)	{
		r = mod(r,2.0);
		r = (r > 1.0) ? 2.0 - r : r;	
	}
	
	if (r <= 1.0)	{
		
		a = mod(a + angleShift, 1.0);
		vec2	pt = vec2(a,r);
		//returnMe = vec4(r,a,0.0,1.0);
		
		if (mirrorX)	{
			pt.x = (pt.x < 0.5) ? pt.x * 2.0 : 2.0 - pt.x * 2.0;
		}
		if (mirrorY)	{
			pt.y = (pt.y < 0.5) ? pt.y * 2.0 : 2.0 - pt.y * 2.0;	
		}
		
		pt = rotatePointNorm(pt,preRotateAngle);
		returnMe = IMG_NORM_PIXEL(inputImage,pt);
		
		if (repeatStyle > 0)	{
			if (repeatStyle == 1)
				returnMe.a *= (val > 1.0) ? 1.0 - repeatDecay * (val) : 1.0;
			else	{
				float repCount = 1.0 / (1.0+resultSize);
				returnMe.a *= 1.0 - repeatDecay * floor(val) / repCount;
			}
		}
	}
	
	gl_FragColor = returnMe;
}
