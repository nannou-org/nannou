// ------------volumetric light----------- //
vec3 evaluateLight(in vec3 pos)
{
    float distanceToL = length(L1-pos);
    float distanceToL2 = length(L2-pos);
    return (
          lightCol2 * 1.0/(distanceToL*distanceToL)
           +lightCol1 * 1.0/(distanceToL2*distanceToL2)
          )*.5;
}