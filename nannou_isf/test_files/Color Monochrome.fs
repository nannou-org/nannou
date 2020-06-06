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
            "DEFAULT": 1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "intensity",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0.6,
                0.45,
                0.3,
                1
            ],
            "NAME": "color",
            "TYPE": "color"
        }
    ],
    "ISFVSN": "2"
}
*/


//const vec4		lumcoeff = vec4(0.299, 0.587, 0.114, 0.0);
const vec4 	lumcoeff = vec4(0.2126, 0.7152, 0.0722, 0.0);

void main() {
	vec4		srcPixel = IMG_THIS_PIXEL(inputImage);
	float		luminance = dot(srcPixel,lumcoeff);
	//float		luminance = (srcPixel.r + srcPixel.g + srcPixel.b)/3.0;
	vec4		dstPixel = (luminance >= 0.50)
		? mix(color, vec4(1,1,1,srcPixel.a), (luminance-0.5)*2.0)
		: mix(vec4(0,0,0,srcPixel.a), color, luminance*2.0);
	gl_FragColor = mix(srcPixel, dstPixel, intensity);
}