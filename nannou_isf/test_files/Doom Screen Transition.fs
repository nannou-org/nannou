/*{
    "CATEGORIES": [
        "Wipe"
    ],
    "CREDIT": "Automatically converted from https://www.github.com/gl-transitions/gl-transitions/tree/master/DoomScreenTransition.glsl",
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
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "frequency",
            "TYPE": "float"
        },
        {
            "NAME": "noise",
            "TYPE": "float"
        },
        {
            "DEFAULT": 50,
            "MAX": 100,
            "MIN": 0,
            "NAME": "bars",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "dripScale",
            "TYPE": "float"
        },
        {
            "DEFAULT": 2,
            "MAX": 10,
            "MIN": 0,
            "NAME": "amplitude",
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



// Author: Zeh Fernando
// License: MIT


// Transition parameters --------

// Number of total bars/columns

// Multiplier for speed ratio. 0 = no variation when going down, higher = some elements go much faster

// Further variations in speed. 0 = no noise, 1 = super noisy (ignore frequency)

// Speed variation horizontally. the bigger the value, the shorter the waves

// How much the bars seem to "run" from the middle of the screen first (sticking to the sides). 0 = no drip, 1 = curved drip


// The code proper --------

float rand(int num) {
  return fract(mod(float(num) * 67123.313, 12.0) * sin(float(num) * 10.3) * cos(float(num)));
}

float wave(int num) {
  float fn = float(num) * frequency * 0.1 * float(bars);
  return cos(fn * 0.5) * cos(fn * 0.13) * sin((fn+10.0) * 0.3) / 2.0 + 0.5;
}

float drip(int num) {
  return sin(float(num) / float(bars - 1) * 3.141592) * dripScale;
}

float pos(int num) {
  return (noise == 0.0 ? wave(num) : mix(wave(num), rand(num), noise)) + (dripScale == 0.0 ? 0.0 : drip(num));
}

vec4 transition(vec2 uv) {
  int bar = int(uv.x * (float(bars)));
  float scale = 1.0 + pos(bar) * amplitude;
  float phase = progress * scale;
  float posY = uv.y / vec2(1.0).y;
  vec2 p;
  vec4 c;
  if (phase + posY < 1.0) {
    p = vec2(uv.x, uv.y + mix(0.0, vec2(1.0).y, phase)) / vec2(1.0).xy;
    c = getFromColor(p);
  } else {
    p = uv.xy / vec2(1.0).xy;
    c = getToColor(p);
  }

  // Finally, apply the color
  return c;
}



void main()	{
	gl_FragColor = transition(isf_FragNormCoord.xy);
}