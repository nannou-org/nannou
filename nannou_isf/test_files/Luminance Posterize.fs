/*{
    "CATEGORIES": [
        "Color Effect",
        "Retro"
    ],
    "CREDIT": "by zoidberg",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 30,
            "MAX": 30,
            "MIN": 2,
            "NAME": "levels",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

vec3 rgb2hsv(vec3 c);
vec3 hsv2rgb(vec3 c);

void main() {
	//	get the src pixel, convert to HSL, posterize the 'L', convert back to RGB
	
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	vec4		tmpColor;
	tmpColor.xyz = rgb2hsv(srcPixel.rgb);
	float		amountPerLevel = 1.0/(levels);
	float		numOfLevels = floor(tmpColor.z/amountPerLevel);
	tmpColor.z = numOfLevels*(1.0/(levels-1.0));
	gl_FragColor.rgb = hsv2rgb(tmpColor.xyz);
	gl_FragColor.a = srcPixel.a;
	
}


vec3 rgb2hsv(vec3 c)	{
	vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	//vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
	//vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
	vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
	vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);
	
	float d = q.x - min(q.w, q.y);
	float e = 1.0e-10;
	return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)	{
	vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
	vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
	return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}