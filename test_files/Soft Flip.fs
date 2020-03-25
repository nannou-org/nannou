/*{
    "CATEGORIES": [
        "Geometry Adjustment"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": -0.25,
            "MAX": 1,
            "MIN": -1,
            "NAME": "angle",
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
            "NAME": "centerPt",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": 0.1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "lineWidth",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "NAME": "flipH",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 1,
            "NAME": "flipV",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2"
}
*/



const float pi = 3.14159265359;


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

//	returns the distance from pt0 to the line defined by pt1 and pt2
float distancePtToLine(vec2 pt1, vec2 pt2, vec2 pt0)	{
	return ((pt2.y-pt1.y)*pt0.x-(pt2.x-pt1.x)*pt0.y+pt2.x*pt1.y-pt2.y*pt1.x)/(sqrt(pow(pt2.y-pt1.y,2.0)+pow(pt2.x-pt1.x,2.0)));
}

void main()	{
	vec2		loc = isf_FragNormCoord;
	vec4		returnMe = vec4(vec3(0.0),1.0);
	vec2		p1 = centerPt;
	vec2		p2 = p1 + vec2(cos(angle*pi),sin(angle*pi));
	float		val = distancePtToLine(p1,p2,loc);
	vec2		flipLoc = loc;
	flipLoc.x = (flipH) ? 1.0 - flipLoc.x : flipLoc.x;
	flipLoc.y = (flipV) ? 1.0 - flipLoc.y : flipLoc.y;

	if (abs(val) < lineWidth)	{
		vec4	pix1 = IMG_NORM_PIXEL(inputImage,loc);
		vec4	pix2 = IMG_NORM_PIXEL(inputImage,flipLoc);
		returnMe = mix(pix1,pix2,((-val+lineWidth)) / (2.0*lineWidth));
		//returnMe.r = 2.0 * ((val+lineWidth)) / (2.0*lineWidth);
		//returnMe.g = 2.0 - 2.0 * ((val+lineWidth)) / (2.0*lineWidth);
	}
	else if (val > 0.0)	{
		returnMe = IMG_NORM_PIXEL(inputImage,loc);
	}
	else	{
		returnMe = IMG_NORM_PIXEL(inputImage,flipLoc);
	}
	gl_FragColor = returnMe;
}
