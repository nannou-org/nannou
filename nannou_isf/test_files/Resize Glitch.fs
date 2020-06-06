/*{
    "CATEGORIES": [
        "Glitch",
        "Geometry Adjustment"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "randomFrequency",
            "TYPE": "float"
        },
        {
            "NAME": "glitchNow",
            "TYPE": "event"
        },
        {
            "DEFAULT": 2,
            "MAX": 10,
            "MIN": 0.01,
            "NAME": "levelX",
            "TYPE": "float"
        },
        {
            "DEFAULT": 2,
            "MAX": 10,
            "MIN": 0.01,
            "NAME": "levelY",
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
            "NAME": "center",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": true,
            "NAME": "randomizeWidth",
            "TYPE": "bool"
        },
        {
            "DEFAULT": true,
            "NAME": "randomizeHeight",
            "TYPE": "bool"
        },
        {
            "DEFAULT": true,
            "NAME": "randomizeCenter",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2"
}
*/

float random (vec2 st) {
    return fract(sin(dot(st.xy,vec2(12.9898,78.233)))*43758.5453123);
}

void main() {
	vec2		loc;
	vec2		modifiedCenter;
	
	loc = isf_FragNormCoord;
	modifiedCenter = (randomizeCenter) ? vec2(random(vec2(TIME*1.24,0.234)),random(vec2(TIME*2.93,1.234))) : center;
	
	float		newWidth = 1.0;
	float		newHeight = 1.0;
	
	bool		doGlitch = false;
	
	if (glitchNow)	{
		doGlitch = true;
	}
	else if (randomFrequency == 1.0)	{
		doGlitch = true;
	}
	else	{
		float	val = random(vec2(TIME,0.2321));
		if (val <= randomFrequency)	{
			doGlitch = true;
		}
	}
	
	if (doGlitch == true)	{
		newWidth = (randomizeWidth == false) ? levelX : levelX * random(vec2(TIME+0.315,FRAMEINDEX+32));
		newHeight = (randomizeHeight == false) ? levelY : levelY * random(vec2(TIME+0.942,FRAMEINDEX+43));
	}
	
	loc.x = (loc.x - modifiedCenter.x)*(1.0/newWidth) + modifiedCenter.x;
	loc.y = (loc.y - modifiedCenter.y)*(1.0/newHeight) + modifiedCenter.y;
	if ((loc.x < 0.0)||(loc.y < 0.0)||(loc.x > 1.0)||(loc.y > 1.0))	{
		gl_FragColor = vec4(0.0);
	}
	else	{
		gl_FragColor = IMG_NORM_PIXEL(inputImage,loc);
	}
}
