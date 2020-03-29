/*{
    "CATEGORIES": [
        "Utility"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "Shows the current time of day or time since the composition started",
    "INPUTS": [
        {
            "DEFAULT": [
                1,
                0.5899132640831176,
                0,
                1
            ],
            "LABEL": "Color",
            "NAME": "colorInput",
            "TYPE": "color"
        },
        {
            "DEFAULT": 0,
            "LABEL": "Clock Mode",
            "LABELS": [
                "Time",
                "Countdown",
                "Counter"
            ],
            "NAME": "clockMode",
            "TYPE": "long",
            "VALUES": [
                0,
                1,
                2
            ]
        },
        {
            "DEFAULT": 0,
            "LABEL": "Y Offset",
            "MAX": 1,
            "MIN": -1,
            "NAME": "yOffset",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "LABEL": "Blink",
            "NAME": "blinkingColons",
            "TYPE": "bool"
        },
        {
            "LABEL": "24 Hour",
            "NAME": "twentyFourHourStyle",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
        }
    ]
}
*/



float segment(vec2 uv, bool On) {
	return (On) ?  (1.0 - smoothstep(0.05,0.15,abs(uv.x))) *
			       (1.-smoothstep(0.35,0.55,abs(uv.y)+abs(uv.x)))
		        : 0.;
}

float digit(vec2 uv,int num) {
	float seg= 0.;
    seg += segment(uv.yx+vec2(-1., 0.),num!=-1 && num!=1 && num!=4                    );
	seg += segment(uv.xy+vec2(-.5,-.5),num!=-1 && num!=1 && num!=2 && num!=3 && num!=7);
	seg += segment(uv.xy+vec2( .5,-.5),num!=-1 && num!=5 && num!=6                    );
   	seg += segment(uv.yx+vec2( 0., 0.),num!=-1 && num!=0 && num!=1 && num!=7          );
	seg += segment(uv.xy+vec2(-.5, .5),num==0 || num==2 || num==6 || num==8           );
	seg += segment(uv.xy+vec2( .5, .5),num!=-1 && num!=2                              );
    seg += segment(uv.yx+vec2( 1., 0.),num!=-1 && num!=1 && num!=4 && num!=7          );	
	return seg;
}

float showNum(vec2 uv,int nr, bool zeroTrim) { // nr: 2 digits + sgn . zeroTrim: trim leading "0"
	if (abs(uv.x)>2.*1.5 || abs(uv.y)>1.2) return 0.;

	if (nr<0) {
		nr = -nr;
		if (uv.x>1.5) {
			uv.x -= 2.;
			return segment(uv.yx,true); // minus sign.
		}
	}
	
	if (uv.x>0.) {
		nr /= 10; if (nr==0 && zeroTrim) nr = -1;
		uv -= vec2(.75,0.);
	} else {
		uv += vec2(.75,0.); 
		nr = int(mod(float(nr),10.));
	}

	return digit(uv,nr);
}

float colon(vec2 uv, vec2 cCenter, float cRadius)	{
	float		returnMe = distance(uv,cCenter);
	if (returnMe > cRadius)
		returnMe = 0.0;
	else
		returnMe = 1.0 - pow(returnMe / cRadius,4.0);
	return returnMe;
}



//	a simplfied version of the number drawing from http://www.interactiveshaderformat.com/sketches/120




void main()	{
	vec4		returnMe = vec4(0.0);
	vec2		uv = isf_FragNormCoord;
	float		adjustedOffset = (yOffset*1.2*(RENDERSIZE.y/RENDERSIZE.x));
	vec2		loc = uv;
	loc.y = loc.y - adjustedOffset;	
	
	//	The first element of the vector is the year, the second element is the month,
	//	the third element is the day, and the fourth element is the time (in seconds) within the day.
	vec4		currentDate = DATE;
	if (clockMode == 1)
		currentDate.a = 86400.0 - currentDate.a;
	else if (clockMode == 2)
		currentDate = vec4(TIME);
	
	float		tmpVal = currentDate.a;
	float		h = 0.0;
	float		m = 0.0;
	float		s = 0.0;

	s = mod(tmpVal,60.0);
	tmpVal = tmpVal / 60.0;
	m = mod(tmpVal,60.0);
	tmpVal = tmpVal / 60.0;
	h = mod(tmpVal,60.0);
	if ((!twentyFourHourStyle)&&(clockMode == 0))	{
		h = mod(h,12.0);
		if (h < 1.0)
			h = 12.0;
	}
	
	float		seg = 0.0;
	int			displayTime = 0;
	
	if (loc.x < 0.3)	{
		loc.x = 1.0 - (loc.x + 0.37);
		loc = (loc * 3.0 - 1.5) * 4.0;
		displayTime = int(h);
	}
	else if (loc.x < 0.6)	{
		loc.x = 1.0 - (loc.x+0.05);
		loc = (loc * 3.0 - 1.5) * 4.0;
		displayTime = int(m);
	}
	else	{
		loc.x = 1.0 - (loc.x - 0.3);
		loc = (loc * 3.0 - 1.5) * 4.0;
		displayTime = int(s);
	}

	seg = showNum(loc,displayTime,false);
	
	if ((!blinkingColons)||(mod(currentDate.a,1.0)<0.5))	{
		seg += colon(uv,vec2(0.293,0.53+adjustedOffset),0.015);
		seg += colon(uv,vec2(0.293,0.47+adjustedOffset),0.015);
	
		seg += colon(uv,vec2(0.633,0.53+adjustedOffset),0.015);
		seg += colon(uv,vec2(0.633,0.47+adjustedOffset),0.015);
	}
	if (seg > 0.0)
		returnMe = colorInput * seg;
	
	gl_FragColor = returnMe;
}
