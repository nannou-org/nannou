/*{
    "CATEGORIES": [
        "Feedback"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": 0,
            "NAME": "maskRadius",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 16,
            "MIN": 0,
            "NAME": "feedbackRate",
            "TYPE": "float"
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
            "MAX": 0.25,
            "MIN": -0.25,
            "NAME": "twirlAmount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "fadeRate",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "centerFeedback",
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
            "NAME": "feedbackCenter",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": 2,
            "LABELS": [
                "Mask",
                "CenteredMask",
                "Scaled",
                "Wrap",
                "MirrorWrap",
                "InvertedMask"
            ],
            "NAME": "styleMode",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2,
                3,
                4,
                5
            ]
        },
        {
            "NAME": "clearBuffer",
            "TYPE": "event"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "PERSISTENT": true,
            "TARGET": "feedbackBuffer"
        }
    ]
}
*/

const float pi = 3.1415926535897932384626433832795;
const float tau =  6.2831853071795864769252867665590;

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
	vec4		inputPixelColor = vec4(0.0);
	vec2		loc = isf_FragNormCoord.xy;
	loc -= (feedbackCenter - vec2(0.5));
	loc = mix(vec2((loc.x*RENDERSIZE.x/RENDERSIZE.y)-(RENDERSIZE.x*.5-RENDERSIZE.y*.5)/RENDERSIZE.y,loc.y), 
				vec2(loc.x,loc.y*(RENDERSIZE.y/RENDERSIZE.x)-(RENDERSIZE.y*.5-RENDERSIZE.x*.5)/RENDERSIZE.x), 
				step(RENDERSIZE.x,RENDERSIZE.y));
	
	float		val1 = shapeForType(loc,shape1);
	float		val2 = shapeForType(loc,shape2);
	vec2		locCenter = feedbackCenter * RENDERSIZE;
	loc = gl_FragCoord.xy;
	float		val = mix(val1,val2,mixPoint);
	float		a1 = (atan(locCenter.y-loc.y,locCenter.x-loc.x) + pi) / (tau);
	
	val += (shapeWobble == 0.0) ? 0.0 : shapeWobble * ((sin(TIME+10.0*tau*(a1)))/13.0 + (sin(-TIME*2.1+17.0*tau*(a1)))/17.0 + (sin(19.0*tau*(a1)))/19.0);
	
	float		scaledRadius = maskRadius * min(RENDERSIZE.x,RENDERSIZE.y);
	float		dist = val * min(RENDERSIZE.x,RENDERSIZE.y);
	bool		invertMask = (styleMode == 5);
	float		feedbackLevel = 1.0;
	
	//	if within the shape, just do the shape
	if (((dist>scaledRadius)&&(invertMask))||((dist<scaledRadius)&&(invertMask==false)))	{
		if (styleMode == 1)	{
			loc -= (locCenter-RENDERSIZE/2.0);
		}
		else if (styleMode == 2)	{
			loc -= (locCenter-RENDERSIZE/2.0);
			loc -= RENDERSIZE/2.0;
			loc /= (2.0 * maskRadius);
			loc += RENDERSIZE/2.0;
		}
		else if (styleMode == 3)	{
			float	r = distance(locCenter,loc);
			float 	a = atan((loc.y-locCenter.y),(loc.x-locCenter.x));	
			a = (a + pi)/(tau);
			a = mod(a+0.75,1.0);
			r /= scaledRadius;
			loc = RENDERSIZE * vec2(a,r);
		}
		else if (styleMode == 4)	{
			float	r = distance(locCenter,loc);
			float 	a = atan((loc.y-locCenter.y),(loc.x-locCenter.x));	
			a = (a + pi)/(tau);
			a = mod(a+0.75,1.0);
			a = (a < 0.5) ? a * 2.0 : 2.0 - a * 2.0;
			r /= scaledRadius;
			loc = RENDERSIZE * vec2(a,r);
		}
		inputPixelColor = IMG_PIXEL(inputImage,loc);
		feedbackLevel = centerFeedback;
		/*
		if ((centerFeedback > 0.0)&&(clearBuffer == false))	{
			inputPixelColor = mix(inputPixelColor,IMG_THIS_PIXEL(feedbackBuffer),centerFeedback);	
		}
		*/
		//inputPixelColor = vec4(loc.x,loc.y,0.0,1.0);
	}
	if ((clearBuffer == false)&&(feedbackLevel > 0.0))	{
		//float	r = distance(RENDERSIZE/2.0,loc);
		float 	a = atan((loc.y-locCenter.y),(loc.x-locCenter.x));
		//a = tau * (floor(a * 5.0 / tau) / 5.0);
		float	shiftAmount = -1.0 * feedbackRate;
		vec2	shift = shiftAmount * vec2(cos(a + twirlAmount * tau), sin(a + twirlAmount * tau));
		loc = (invertMask) ? loc - shift : loc + shift;
		inputPixelColor = mix(inputPixelColor,IMG_PIXEL(feedbackBuffer,loc),feedbackLevel);
		inputPixelColor.a -= fadeRate / 50.0;
		//inputPixelColor = vec4((a+pi)/(2.0*pi),2.0*r/RENDERSIZE.x,0.0,1.0);
		//inputPixelColor = vec4(shift.x*2.0-1.0,shift.y*2.0-1.0,0.0,1.0);
	}
	
	gl_FragColor = inputPixelColor;
}
