
/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Glitch", "Retro"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "noiseLevel",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.5
		},
		{
			"NAME": "distortion1",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 5.0,
			"DEFAULT": 1.0
		},
		{
			"NAME": "distortion2",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 5.0,
			"DEFAULT": 5.0
		},
		{
			"NAME": "speed",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.3
		},
		{
			"NAME": "scroll",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "scanLineThickness",
			"TYPE": "float",
			"MIN": 1.0,
			"MAX": 50.0,
			"DEFAULT": 25.0
		},
		{
			"NAME": "scanLineIntensity",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.5
		},
		{
			"NAME": "scanLineOffset",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		}		
	]
}*/

//	Adapted from http://www.airtightinteractive.com/demos/js/badtvshader/js/BadTVShader.js
//	Also uses adopted Ashima WebGl Noise: https://github.com/ashima/webgl-noise

/*
 * The MIT License
 * 
 * Copyright (c) 2014 Felix Turner
 * 
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 * 
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 * 
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 * 
*/

		
// Start Ashima 2D Simplex Noise

const vec4 C = vec4(0.211324865405187,0.366025403784439,-0.577350269189626,0.024390243902439);

vec3 mod289(vec3 x) {
	return x - floor(x * (1.0 / 289.0)) * 289.0;
}

vec2 mod289(vec2 x) {
	return x - floor(x * (1.0 / 289.0)) * 289.0;
}

vec3 permute(vec3 x) {
	return mod289(((x*34.0)+1.0)*x);
}

float snoise(vec2 v)	{
	vec2 i  = floor(v + dot(v, C.yy) );
	vec2 x0 = v -   i + dot(i, C.xx);

	vec2 i1;
	i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
	vec4 x12 = x0.xyxy + C.xxzz;
	x12.xy -= i1;

	i = mod289(i); // Avoid truncation effects in permutation
	vec3 p = permute( permute( i.y + vec3(0.0, i1.y, 1.0 ))+ i.x + vec3(0.0, i1.x, 1.0 ));

	vec3 m = max(0.5 - vec3(dot(x0,x0), dot(x12.xy,x12.xy), dot(x12.zw,x12.zw)), 0.0);
	m = m*m ;
	m = m*m ;

	vec3 x = 2.0 * fract(p * C.www) - 1.0;
	vec3 h = abs(x) - 0.5;
	vec3 ox = floor(x + 0.5);
	vec3 a0 = x - ox;

	m *= 1.79284291400159 - 0.85373472095314 * ( a0*a0 + h*h );

	vec3 g;
	g.x  = a0.x  * x0.x  + h.x  * x0.y;
	g.yz = a0.yz * x12.xz + h.yz * x12.yw;
	return 130.0 * dot(m, g);
}

// End Ashima 2D Simplex Noise

const float tau = 6.28318530718;

//	use this pattern for scan lines

vec2 pattern(vec2 pt) {
	float s = 0.0;
	float c = 1.0;
	vec2 tex = pt * RENDERSIZE;
	vec2 point = vec2( c * tex.x - s * tex.y, s * tex.x + c * tex.y ) * (1.0/scanLineThickness);
	float d = point.y;

	return vec2(sin(d + scanLineOffset * tau + cos(pt.x * tau)), cos(d + scanLineOffset * tau + sin(pt.y * tau)));
}

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

void main() {
	vec2 p = isf_FragNormCoord;
	float ty = TIME*speed;
	float yt = p.y - ty;

	//smooth distortion
	float offset = snoise(vec2(yt*3.0,0.0))*0.2;
	// boost distortion
	offset = pow( offset*distortion1,3.0)/max(distortion1,0.001);
	//add fine grain distortion
	offset += snoise(vec2(yt*50.0,0.0))*distortion2*0.001;
	//combine distortion on X with roll on Y
	vec2 adjusted = vec2(fract(p.x + offset),fract(p.y-scroll) );
	vec4 result = IMG_NORM_PIXEL(inputImage, adjusted);
	vec2 pat = pattern(adjusted);
	vec3 shift = scanLineIntensity * vec3(0.3 * pat.x, 0.59 * pat.y, 0.11) / 2.0;
	result.rgb = (1.0 + scanLineIntensity / 2.0) * result.rgb + shift + (rand(adjusted * TIME) - 0.5) * noiseLevel;
	gl_FragColor = result;

}