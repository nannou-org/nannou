float map( float value, float inMin, float inMax, float outMin, float outMax ) 
{
    return ( (value - inMin) / ( inMax - inMin ) * ( outMax - outMin ) ) + outMin; 
}