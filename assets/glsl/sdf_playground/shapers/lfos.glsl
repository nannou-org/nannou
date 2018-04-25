//#pragma include "../util/pi.glsl"

//--------- LFO's --------------
// FLOATS
float tri(float x) {
    return asin(sin(x))/(PI/2.);
}
float pulse(float x) {
    return (floor(sin(x))+0.5)*2.;
}
float saw(float x) {
    return (fract((x/2.)/PI)-0.5)*2.;
}
float noise(float x) {
    return (fract(sin((x*2.) *(12.9898+78.233)) * 43758.5453)-0.5)*2.;
}
// VEC3
vec3 tri(vec3 x) {
    return vec3(asin(sin(x))/(PI/2.));
}
vec3 pulse(vec3 x) {
    return vec3((floor(sin(x))+0.5)*2.);
}
vec3 saw(vec3 x) {
    return vec3((fract((x/2.)/PI)-0.5)*2.);
}
vec3 noise(vec3 x) {
    return vec3((fract(sin((x*2.) *(12.9898+78.233)) * 43758.5453)-0.5)*2.);
}


float lfo(int type, float x){
    if(type == 0) return sin(x);
    else if(type == 1) return tri(x);
    else if(type == 2) return saw(x);
    else if(type == 3) return pulse(x);
    else if(type == 4) return noise(x);
    else return 0.0;
}
vec3 lfo(int type, vec3 x){
    if(type == 0) return cos(x);
    else if(type == 1) return tri(x);
    else if(type == 2) return saw(x);
    else if(type == 3) return pulse(x);
    else if(type == 4) return noise(x);
    else return vec3(0.0);
}
