/*{
    "CATEGORIES": [
        "Wipe"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/undulatingBurnOut.glsl",
    "DESCRIPTION": "",
    "INPUTS": [
        {
            "NAME": "startImage",
            "TYPE": "image"
        },
        {
            "NAME": "endImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "progress",
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
            "NAME": "center",
            "TYPE": "point2D"
        },
        {
            "DEFAULT": [
                0,
                0,
                0,
                1
            ],
            "NAME": "color",
            "TYPE": "color"
        },
        {
            "DEFAULT": 0.03,
            "MAX": 1,
            "MIN": 0,
            "NAME": "smoothness",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/



vec4 getFromColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(startImage, inUV);
}
vec4 getToColor(vec2 inUV)	{
	return IMG_NORM_PIXEL(endImage, inUV);
}



// License: MIT
// Author: pthrasher
// adapted by gre from https://gist.github.com/pthrasher/8e6226b215548ba12734


const float M_PI = 3.14159265358979323846;

float quadraticInOut(float t) {
  float p = 2.0 * t * t;
  return t < 0.5 ? p : -p + (4.0 * t) - 1.0;
}

float getGradient(float r, float dist) {
  float d = r - dist;
  return mix(
    smoothstep(-smoothness, 0.0, r - dist * (1.0 + smoothness)),
    -1.0 - step(0.005, d),
    step(-0.005, d) * step(d, 0.01)
  );
}

float getWave(vec2 p){
  vec2 _p = p - center; // offset from center
  float rads = atan(_p.y, _p.x);
  float degs = degrees(rads) + 180.0;
  vec2 range = vec2(0.0, M_PI * 30.0);
  vec2 domain = vec2(0.0, 360.0);
  float ratio = (M_PI * 30.0) / 360.0;
  degs = degs * ratio;
  float x = progress;
  float magnitude = mix(0.02, 0.09, smoothstep(0.0, 1.0, x));
  float offset = mix(40.0, 30.0, smoothstep(0.0, 1.0, x));
  float ease_degs = quadraticInOut(sin(degs));
  float deg_wave_pos = (ease_degs * magnitude) * sin(x * offset);
  return x + deg_wave_pos;
}

vec4 transition(vec2 p) {
  float dist = distance(center, p);
  float m = getGradient(getWave(p), dist);
  vec4 cfrom = getFromColor(p);
  vec4 cto = getToColor(p);
  return mix(mix(cfrom, cto, m), mix(cfrom, color, 0.75), step(m, -2.0));
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}