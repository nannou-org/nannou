/*{
    "CATEGORIES": [
        "Masking"
    ],
    "CREDIT": "by VIDVOX",
    "DESCRIPTION": "Pixels update only if within range of the specified lines to create a slit scan style",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 1,
            "MAX": 1.5,
            "MIN": 0,
            "NAME": "spacing",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.33,
            "MAX": 1,
            "MIN": 0,
            "NAME": "line_width",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": -1,
            "NAME": "angle",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "shift",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "edgeSharpness",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/


const float pi = 3.14159265359;


float pattern() {
	float s = sin(angle * pi);
	float c = cos(angle * pi);
	vec2 tex = isf_FragNormCoord * RENDERSIZE;
	float spaced = length(RENDERSIZE) * spacing;
	vec2 point = vec2( c * tex.x - s * tex.y, s * tex.x + c * tex.y ) * max(1.0/spaced,0.001);
	float d = point.y;
	float w = line_width;
	if (w > spacing)	{
		w = 0.99*spacing;	
	}
	return ( mod(d + shift*spacing + w * 0.5,spacing) );
}

void main()
{
	vec4	freshPixel = IMG_PIXEL(inputImage,gl_FragCoord.xy);
	//	If we're on the line, update, otherwise use the stale pixel
	//vec4 	result = IMG_PIXEL(bufferVariableNameA,gl_FragCoord.xy);
	vec4 result = vec4(0.0);
	float pat = pattern();
	float w = line_width;
	if (w > spacing)	{
		w = 0.99*spacing;	
	}

	if ((pat > 0.0)&&(pat <= w))	{
		float percent = (1.0-abs(w-2.0*pat)/w);
		percent = clamp(percent,0.0,1.0);
		percent = mix(percent, pow(percent,1.0/5.0), edgeSharpness);
		result = mix(result, freshPixel, percent);
		//result = vec4(percent,percent,percent,1.0);
		//result = clamp(result+edgeSharpness, 0.0, 1.0);
	}
	gl_FragColor = result;
}
