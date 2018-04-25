const vec3 cGammaCorrection = vec3( 0.4545454545 );

vec3 gamma( in vec3 color )
{
  return pow( color, cGammaCorrection );
}

vec4 gamma( in vec4 color )
{
  return vec4( gamma( color.rgb ), color.a );
}
