/*{
    "CATEGORIES": [
        "Color"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "DEFAULT": 0.5,
            "MAX": 1,
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
            "NAME": "shift",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": [
                1,
                0.5,
                0,
                1
            ],
            "NAME": "xcolor",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0,
                0.5,
                1,
                1
            ],
            "NAME": "ycolor",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0,
                0,
                0,
                0
            ],
            "NAME": "background",
            "TYPE": "color"
        }
    ],
    "ISFVSN": "2"
}
*/



//	Basically just uses the same gradient as the Sine Warp Tile but uses the x/y values as the mix amounts for our colors


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

	gl_FragColor = background + pat.x * xcolor + pat.y * ycolor;
}