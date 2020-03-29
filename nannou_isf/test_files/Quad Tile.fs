/*{
    "CATEGORIES": [
        "Tile Effect"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 2,
            "MIN": 0,
            "NAME": "size",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "rotation",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "angle",
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
            "NAME": "slide1",
            "TYPE": "point2D"
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
            "NAME": "slide2",
            "TYPE": "point2D"
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
            "NAME": "shift",
            "TYPE": "point2D"
        }
    ],
    "ISFVSN": "2"
}
*/


const float tau = 6.28318530718;


vec2 pattern() {
	float s = sin(tau * rotation);
	float c = cos(tau * rotation);
	vec2 tex = isf_FragNormCoord * RENDERSIZE;
	float scale = 1.0 / max(size,0.001);
	vec2 point = vec2( c * tex.x - s * tex.y, s * tex.x + c * tex.y ) * scale;
	point = (point - shift*RENDERSIZE) / RENDERSIZE;
	//	do this to repeat
	point = mod(point,1.0);
	if (point.x < 0.5)	{
		point.y = mod(point.y + slide1.y, 1.0);
	}
	else	{
		point.y = mod(point.y + slide2.y, 1.0);
	}
	if (point.y < 0.5)	{
		point.x = mod(point.x + slide1.x, 1.0);
	}
	else	{
		point.x = mod(point.x + slide2.x, 1.0);
	}
	//	do this for relections
	point = 1.0-abs(1.0-2.0*point);
	
	//	Now let's do a squish based on angle
	//	convert to polar coordinates
	vec2 center = vec2(0.5,0.5);
	float r = distance(center, point);
	float a = atan ((point.y-center.y),(point.x-center.x));
	
	s = sin(a + tau * angle);
	c = cos(a + tau * angle);
	
	float zoom = RENDERSIZE.x / RENDERSIZE.y;
	
	point.x = (r * c)/zoom + 0.5;
	point.y = (r * s)/zoom + 0.5;
	
	return point;
}


void main() {

	vec2 pat = pattern();

	gl_FragColor = IMG_NORM_PIXEL(inputImage,pat);
}