#version 330

/*
//--------- UTILS --------------
#pragma include "util/uniforms.glsl"
#pragma include "../util/pi.glsl"
#pragma include "../util/map.glsl"
#pragma include "../util/matrixOp.glsl"
//--------- RANDOMNESS --------------
#pragma include "../random/random.glsl"
#pragma include "../random/noise2D.glsl"
#pragma include "../random/noise3D.glsl"
#pragma include "../random/noise4D.glsl"
#pragma include "../random/fbm.glsl"
//--------- SHAPING FUNCTIONS --------------
#pragma include "../shapers/lfos.glsl"
#pragma include "../shapers/easings.glsl"
#pragma include "../shapers/easing_lfo.glsl"
//--------- COLOUR --------------
#pragma include "../colour/blackbody.glsl"
#pragma include "../colour/colour_palettes.glsl"
#pragma include "../colour/colourise.glsl"
#pragma include "../colour/gamma.glsl"
#pragma include "../colour/hsb.glsl"
#pragma include "../colour/hue_shift.glsl"
//--------- POST PROCESSING --------------
#pragma include "../post_processing/vignette.glsl"
//--------- SIGNED DISTANCE FUNCTIONS --------------
#pragma include "../sdf/hg_sdf.glsl"
#pragma include "../sdf/ray_march.glsl"
#pragma include "../sdf/shapes2D.glsl"
#pragma include "../sdf/lights.glsl"
*/
/*
// inputs
in vec2 vTexCoord;
// outputs
out vec4 outputColor;

//vec3    id; // unique id per shape
//vec3    h;  // light amount


//--------- CREATE YOUR UNIVERSE --------------
float scene(vec3 p)
{  
    float mind = 1e5;
  
//    mind = fBox(p, 1.0);
    return(mind);
}


//--------- MASTER MAIN FUNCTION --------------
void main()
{    
    //h *= 0.; 
    //float accum = 0.;

    
    // The screen coordinates
    vec2 f = vTexCoord; // Screen coordinates. Range: [0, 1]
    vec2 R = iResolution.xy;
    vec2 uv  = vec2(f-R/2.) / R.y;
    
    float t = iTime;
    vec3 col = vec3(0., 0., 0.);

    vec3    dir = camera(uv);
    vec3    pos = vec3(-.0, .0, 149.);
    
    vec4    inter = (march(pos, dir));

    col.xyz = .5-vec3(.59, .59, .87)*( (inter.w*.006251)+(-.125+inter.x*.006));
    col.xyz += -.5+h*.253061251;
    col.xyz += + accum  * (.750-exp(-0.001*inter.w*inter.w));

    col.xyz = blackbody((15.-inter.y*.06125+.1*inter.x)*100.);
    col = col+inter.y*.001*vec4(.0, 1.2, 1., 1.)*2.;

    
    vec3 col = vec3(1.0,0.0,0.0);

    // Output to screen
    outputColor = vec4(col,1.0);    
}
*/


out vec4 color;
in vec2 vTexCoord;

uniform float iTime;
uniform float iResolution;

void main() {
     color = vec4(abs(sin(abs(vTexCoord.x)+iTime*0.3)), cos(abs(vTexCoord.y)+iTime*0.05), sin(iTime*0.1), 1.0);
}
