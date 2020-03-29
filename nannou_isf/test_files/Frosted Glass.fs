/*{
    "CATEGORIES": [
        "Stylize",
        "Blur"
    ],
    "CREDIT": "geeks3d",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.01,
            "LABEL": "Magnitude",
            "MAX": 0.1,
            "MIN": 0,
            "NAME": "magnitude",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.345,
            "LABEL": "Seed",
            "MAX": 1,
            "MIN": 0,
            "NAME": "seed",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

/*

	adapted from http://www.geeks3d.com/20101228/shader-library-frosted-glass-post-processing-shader-glsl/

*/


const float vx_offset = 0.5;

vec4 spline(float x, vec4 c1, vec4 c2, vec4 c3, vec4 c4, vec4 c5, vec4 c6, vec4 c7, vec4 c8, vec4 c9)
{
	float w1, w2, w3, w4, w5, w6, w7, w8, w9;
	w1 = 0.0;
	w2 = 0.0;
	w3 = 0.0;
	w4 = 0.0;
	w5 = 0.0;
	w6 = 0.0;
	w7 = 0.0;
	w8 = 0.0;
	w9 = 0.0;
	float tmp = x * 8.0;
	if (tmp<=1.0) {
		w1 = 1.0 - tmp;
		w2 = tmp;
	}
	else if (tmp<=2.0) {
		tmp = tmp - 1.0;
		w2 = 1.0 - tmp;
		w3 = tmp;
	}
	else if (tmp<=3.0) {
		tmp = tmp - 2.0;
		w3 = 1.0-tmp;
		w4 = tmp;
	}
	else if (tmp<=4.0) {
		tmp = tmp - 3.0;
		w4 = 1.0-tmp;
		w5 = tmp;
	}
	else if (tmp<=5.0) {
		tmp = tmp - 4.0;
		w5 = 1.0-tmp;
		w6 = tmp;
	}
	else if (tmp<=6.0) {
		tmp = tmp - 5.0;
		w6 = 1.0-tmp;
		w7 = tmp;
	}
	else if (tmp<=7.0) {
		tmp = tmp - 6.0;
		w7 = 1.0 - tmp;
		w8 = tmp;
	}
	else {
		//tmp = saturate(tmp - 7.0);
		// http://www.ozone3d.net/blogs/lab/20080709/saturate-function-in-glsl/
		tmp = clamp(tmp - 7.0, 0.0, 1.0);
		w8 = 1.0-tmp;
		w9 = tmp;
	}
	return w1*c1 + w2*c2 + w3*c3 + w4*c4 + w5*c5 + w6*c6 + w7*c7 + w8*c8 + w9*c9;
}

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

void main()	{
	vec2 uv = isf_FragNormCoord.xy;
	vec4 tc = vec4(1.0, 0.0, 0.0, 0.0);

	float DeltaX = magnitude / 2.0;
	float DeltaY = magnitude / 2.0;
	vec2 ox = vec2(DeltaX,0.0);
	vec2 oy = vec2(0.0,DeltaY);
	vec2 PP = uv - oy;
	vec4 C00 = IMG_NORM_PIXEL(inputImage,PP - ox);
	vec4 C01 = IMG_NORM_PIXEL(inputImage,PP);
	vec4 C02 = IMG_NORM_PIXEL(inputImage,PP + ox);
	PP = uv;
	vec4 C10 = IMG_NORM_PIXEL(inputImage,PP - ox);
	vec4 C11 = IMG_NORM_PIXEL(inputImage,PP);
	vec4 C12 = IMG_NORM_PIXEL(inputImage,PP + ox);
	PP = uv + oy;
	vec4 C20 = IMG_NORM_PIXEL(inputImage,PP - ox);
	vec4 C21 = IMG_NORM_PIXEL(inputImage,PP);
	vec4 C22 = IMG_NORM_PIXEL(inputImage,PP + ox);

	float n = rand(seed*uv);
	n = mod(n, 0.111111)/0.111111;
	vec4 result = spline(n,C00,C01,C02,C10,C11,C12,C20,C21,C22);
	tc = result.rgba;  

	gl_FragColor = tc;
}
