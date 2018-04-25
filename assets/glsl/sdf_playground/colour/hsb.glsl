vec3 rgb2hsb( in vec3 color ){
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(color.bg, K.wz),
                 vec4(color.gb, K.xy),
                 step(color.b, color.g));
    vec4 q = mix(vec4(p.xyw, color.r),
                 vec4(color.r, p.yzx),
                 step(p.x, color.r));
    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)),
                d / (q.x + e),
                q.x);
}

//  Function from Iñigo Quiles
//  https://www.shadertoy.com/view/MsS3Wc
vec3 hsb2rgb( in vec3 color ){
    vec3 rgb = clamp(abs(mod(color.x*6.0+vec3(0.0,4.0,2.0),
                             6.0)-3.0)-1.0,
                     0.0,
                     1.0 );
    rgb = rgb*rgb*(3.0-2.0*rgb);
    return color.z * mix(vec3(1.0), rgb, color.y);
}

