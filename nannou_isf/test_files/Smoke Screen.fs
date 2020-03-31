/*
{
  "CATEGORIES" : [
    "Stylize"
  ],
  "DESCRIPTION" : "A smoke screen overlay effect",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "smokeColor",
      "TYPE" : "color",
      "DEFAULT" : [
        0.75,
        0.75,
        0.75,
        0.75
      ]
    },
    {
      "NAME" : "smokeIntensity",
      "TYPE" : "float",
      "MAX" : 10,
      "DEFAULT" : 1,
      "MIN" : 0
    },
    {
      "NAME" : "smokeDirection",
      "TYPE" : "point2D",
      "MAX" : [
        1,
        1
      ],
      "DEFAULT" : [
        0.5,
        0.75
      ],
      "MIN" : [
        0,
        0
      ]
    }
  ],
  "CREDIT" : "jackdavenport"
}
*/



//	Converted from https://www.shadertoy.com/view/4t2SRz by jackdavenport



float random(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

float noise(vec2 p) {
    
 	return random(vec2(p.x + p.y * 10000.0,p.y + p.x * 10000.0));
    
}

vec2 sw(vec2 p) { return vec2(floor(p.x), floor(p.y)); }
vec2 se(vec2 p) { return vec2(ceil(p.x), floor(p.y)); }
vec2 nw(vec2 p) { return vec2(floor(p.x), ceil(p.y)); }
vec2 ne(vec2 p) { return vec2(ceil(p.x), ceil(p.y)); }

float smoothNoise(vec2 p) {
 
    vec2 interp = smoothstep(0., 1., fract(p));
    float s = mix(noise(sw(p)), noise(se(p)), interp.x);
    float n = mix(noise(nw(p)), noise(ne(p)), interp.x);
    return mix(s, n, interp.y);
    
}

float fractalNoise(vec2 p) {
 
    float n = 0.;
    n += smoothNoise(p);
    n += smoothNoise(p * 2.) / 2.;
    n += smoothNoise(p * 4.) / 4.;
    n += smoothNoise(p * 8.) / 8.;
    n += smoothNoise(p * 16.) / 16.;
    n /= 1. + 1./2. + 1./4. + 1./8. + 1./16.;
    return n;
    
}

void main() {
	vec2	uv = isf_FragNormCoord;
	vec2	sd = (smokeDirection - vec2(0.5));
    vec2	nuv = vec2(uv.x - sd.x * TIME / 3.0, uv.y - sd.y * TIME / 3.0);
    
	float	x = fractalNoise(nuv * 6.0);
	vec4	inputPixel = IMG_NORM_PIXEL(inputImage,uv);
    vec4	final = mix(vec4(x * smokeColor.rgb,max(x,inputPixel.a)), inputPixel, pow(abs(uv.y),pow(smokeColor.a*smokeIntensity*x,2.0)));
    
    gl_FragColor = final;
}

