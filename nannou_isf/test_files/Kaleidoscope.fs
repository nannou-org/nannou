/*{
    "CATEGORIES": [
        "Kaleidoscope",
        "Stylize"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 6,
            "MAX": 32,
            "MIN": 1,
            "NAME": "sides",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": -1,
            "NAME": "angle",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "slidex",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "slidey",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0,
                0
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
        }
    ],
    "ISFVSN": "2"
}
*/


const float tau = 6.28318530718;




void main() {
  // normalize to the center
	vec2 loc = RENDERSIZE * vec2(isf_FragNormCoord[0],isf_FragNormCoord[1]);
	float r = distance(center*RENDERSIZE, loc);
	float a = atan ((loc.y-center.y*RENDERSIZE.y),(loc.x-center.x*RENDERSIZE.x));
	
	// kaleidoscope
	a = mod(a, tau/sides);
	a = abs(a - tau/sides/2.);
	
	loc.x = r * cos(a + tau * angle);
	loc.y = r * sin(a + tau * angle);
	
	loc = (center*RENDERSIZE + loc) / RENDERSIZE;
	
	loc.x = mod(loc.x + slidex, 1.0);
	loc.y = mod(loc.y + slidey, 1.0);

	// sample the image
	if (loc.x < 0.0)	{
		loc.x = mod(abs(loc.x), 1.0);
	}
	if (loc.y < 0.0)	{
		loc.y = mod(abs(loc.y),1.0);
	}
	if (loc.x > 1.0)	{
		loc.x = mod(abs(1.0-loc.x),1.0);
	}
	if(loc.y > 1.0)	{
		loc.y = mod(abs(1.0-loc.y),1.0);	
	}
	gl_FragColor = IMG_NORM_PIXEL(inputImage, loc);;
}