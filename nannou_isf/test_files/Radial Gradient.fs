/*{
    "CATEGORIES": [
        "Color"
    ],
    "CREDIT": "by Carter Rosenberg",
    "INPUTS": [
        {
            "DEFAULT": 0.1,
            "NAME": "radius1",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "NAME": "radius2",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                1,
                0.75,
                0,
                1
            ],
            "NAME": "startColor",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0,
                0.25,
                0.75,
                1
            ],
            "NAME": "endColor",
            "TYPE": "color"
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
            "NAME": "location",
            "TYPE": "point2D"
        }
    ],
    "ISFVSN": "2"
}
*/



void main() {
	vec2 tmpPt = location;
	float mixOffset = distance(tmpPt * RENDERSIZE.x / RENDERSIZE.y, isf_FragNormCoord * RENDERSIZE.x / RENDERSIZE.y);
	float tmpRadius = radius1 + radius2;
	if (mixOffset <= radius1)	{
		gl_FragColor = startColor;
	}
	else if (mixOffset > tmpRadius)	{
		gl_FragColor = endColor;
	}
	else if (radius1 == tmpRadius)	{
		gl_FragColor = endColor;
	}
	else	{
		gl_FragColor = mix(startColor,endColor,(mixOffset-radius1)/(tmpRadius-radius1));
	}
}