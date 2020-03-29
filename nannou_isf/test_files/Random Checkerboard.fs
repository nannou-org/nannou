/*{
    "CATEGORIES": [
        "Pattern", "Color"
    ],
    "CREDIT": "VIDVOX",
    "DESCRIPTION": "Creates a checkerboard pattern with randomized colors",
    "INPUTS": [
        {
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": 0,
            "NAME": "width",
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
            "NAME": "offset",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "hue",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "saturation",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.95,
            "MAX": 1,
            "MIN": 0,
            "NAME": "brightness",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "NAME": "randHue",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0,
            "NAME": "randSaturation",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0,
            "NAME": "randBright",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0,
            "NAME": "randAlpha",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0.71,
            "MAX": 1,
            "MIN": 0,
            "NAME": "rSeed",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

vec4 rand4(vec4 co)	{
	vec4	returnMe = vec4(0.0);
	returnMe.r = rand(co.rg);
	returnMe.g = rand(co.gb);
	returnMe.b = rand(co.ba);
	returnMe.a = rand(co.rb);
	return returnMe;
}

vec3 hsv2rgb(vec3 c)	{
	vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
	vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
	return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {

	vec4 out_color = vec4(0.0);
	vec2 coord = isf_FragNormCoord;
	vec4 hsv = vec4(hue, saturation, brightness, 1.0);
	float minW = max(width,1.0/RENDERSIZE.y);
	vec2 indexes = floor((coord+offset) / minW);
	float index = indexes.x + indexes.y/minW;
	
	hsv.r = (randHue) ? mod(hsv.r + rand(vec2(index, rSeed + 0.34219)),1.0) : hsv.r;
	hsv.g = (randSaturation) ? mod(hsv.g + rand(vec2(index, rSeed + 0.57731)),1.0) : hsv.g;
	hsv.b = (randBright) ? mod(hsv.b + rand(vec2(index, rSeed + 0.79436)),1.0) : hsv.b;
	hsv.a = (randAlpha) ? rand(vec2(hsv.a + index, rSeed + 1.37665)) : hsv.a;
	
	out_color.rgb = hsv2rgb(hsv.rgb);
	out_color.a = hsv.a;
	
	gl_FragColor = out_color;
}