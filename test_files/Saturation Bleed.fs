/*{
    "CATEGORIES": [
        "Color Effect"
    ],
    "CREDIT": "by VIDVOX",
    "DESCRIPTION": "Applies a blur to the saturation levels of pixels.",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "bleedLevel",
            "TYPE": "float"
        },
        {
            "DEFAULT": 2,
            "MAX": 10,
            "MIN": 1,
            "NAME": "depth",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 4,
            "MIN": 0,
            "NAME": "gainLevel",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "HEIGHT": "max(floor($HEIGHT*0.02),1.0)",
            "TARGET": "smaller",
            "WIDTH": "max(floor($WIDTH*0.02),1.0)"
        },
        {
            "HEIGHT": "max(floor($HEIGHT*0.25),1.0)",
            "TARGET": "small",
            "WIDTH": "max(floor($WIDTH*0.25),1.0)"
        },
        {
        }
    ]
}
*/


//	A simple three pass blur – first reduce the size, then do a weighted blur, then do the same thing 
//	but  we are only going to blur the saturation


//	Inspired by Oversaturation glitches
//	https://bavc.github.io/avaa/artifacts/oversaturation.html


#if __VERSION__ <= 120
varying vec2 left_coord;
varying vec2 right_coord;
varying vec2 above_coord;
varying vec2 below_coord;

varying vec2 lefta_coord;
varying vec2 righta_coord;
varying vec2 leftb_coord;
varying vec2 rightb_coord;
#else
in vec2 left_coord;
in vec2 right_coord;
in vec2 above_coord;
in vec2 below_coord;

in vec2 lefta_coord;
in vec2 righta_coord;
in vec2 leftb_coord;
in vec2 rightb_coord;
#endif



vec4 rgb2hsv(vec4 c)	{
	vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	//vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
	//vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
	vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
	vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);
	
	float d = q.x - min(q.w, q.y);
	float e = 1.0e-10;
	return vec4(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x,c.a);
}

vec4 hsv2rgb(vec4 c)	{
	vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
	vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
	return vec4(c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y),c.a);
}




void main()
{
	
	vec4 color = rgb2hsv(IMG_THIS_NORM_PIXEL(inputImage));
	vec4 colorL = rgb2hsv(IMG_NORM_PIXEL(inputImage, left_coord));
	vec4 colorR = rgb2hsv(IMG_NORM_PIXEL(inputImage, right_coord));
	vec4 colorA = rgb2hsv(IMG_NORM_PIXEL(inputImage, above_coord));
	vec4 colorB = rgb2hsv(IMG_NORM_PIXEL(inputImage, below_coord));

	vec4 colorLA = rgb2hsv(IMG_NORM_PIXEL(inputImage, lefta_coord));
	vec4 colorRA = rgb2hsv(IMG_NORM_PIXEL(inputImage, righta_coord));
	vec4 colorLB = rgb2hsv(IMG_NORM_PIXEL(inputImage, leftb_coord));
	vec4 colorRB = rgb2hsv(IMG_NORM_PIXEL(inputImage, rightb_coord));

	vec4 avg = gainLevel * (colorL + colorR + colorA + colorB + colorLA + colorRA + colorLB + colorRB) / 8.0;
	
	
	if (PASSINDEX == 0)	{
		avg = mix(color, (avg + depth)/(1.0+depth), bleedLevel);
		color.g = avg.g;
	}
	else if (PASSINDEX == 1)	{
		vec4 blur = rgb2hsv(IMG_THIS_NORM_PIXEL(smaller));
		avg = mix(color, (avg + depth*blur)/(1.0+depth), bleedLevel);
		color.g = avg.g;
	}
	else if (PASSINDEX == 2)	{
		vec4 blur = rgb2hsv(IMG_THIS_NORM_PIXEL(small));
		avg = mix(color, (avg + depth*blur)/(1.0+depth), bleedLevel);
		color.g = avg.g;
	}
	if (color.g < 0.0)
		color.g = 0.0;
	else if (color.g > 1.0)
		color.g = 1.0;
	color = hsv2rgb(color);
	gl_FragColor = color;
}