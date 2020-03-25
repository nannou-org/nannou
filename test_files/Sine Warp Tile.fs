/*{
    "CATEGORIES": [
        "Kaleidoscope",
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
            "MAX": 0.5,
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
            "NAME": "shift",
            "TYPE": "point2D"
        }
    ],
    "ISFVSN": "2"
}
*/


const float tau = 6.28318530718;


vec2 pattern() {
	float s = sin(tau * rotation * 0.5);
	float c = cos(tau * rotation * 0.5);
	vec2 tex = isf_FragNormCoord;
	float scale = 1.0 / max(size,0.001);
	vec2 point = vec2( c * tex.x - s * tex.y, s * tex.x + c * tex.y ) * scale;
	point = point - scale * shift;
	//	do the sine distort
	point = 0.5 + 0.5 * vec2( sin(scale * point.x), sin(scale * point.y));
	
	//	now do a rotation
	vec2 center = vec2(0.5,0.5);
	float r = distance(center, point);
	float a = atan ((point.y-center.y),(point.x-center.x));
	
	s = sin(a + tau * angle);
	c = cos(a + tau * angle);
	
	float zoom = max(abs(s),abs(c))*RENDERSIZE.x / RENDERSIZE.y;
	
	point.x = (r * c)/zoom + 0.5;
	point.y = (r * s)/zoom + 0.5;

	return point;
}


void main() {

	vec2 pat = pattern();

	gl_FragColor = IMG_NORM_PIXEL(inputImage,pat);
}