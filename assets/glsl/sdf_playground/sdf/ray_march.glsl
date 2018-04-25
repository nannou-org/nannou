//Forward decalre our main scene function that
//we will define in our master shader file
float scene(vec3 p);

//--------- RAY TRACING --------------
vec2 march(vec3 pos, vec3 dir) 
{
    vec2 dist = vec2(0.0, 0.0);
    vec3 p = vec3(0.0, 0.0, 0.0);
    vec2 s = vec2(0.0, 0.0);
    float dinamyceps = E;
    for (float i = -1.; i < I_MAX; ++i)
    {
        p = pos + dir * dist.y;
        dist.x = scene(p);
        dist.y += dist.x*1.;
        dinamyceps = -dist.x+(dist.y)/(1500.);
        // log trick from aiekick
        if (log(dist.y*dist.y/dist.x/1e5)>0. || dist.x < dinamyceps || dist.y > FAR)
        {
            break;
        }
        s.x++;
    }
    s.y = dist.y;
    return (s);
}

float mylength(vec2 p)
{
    float ret;
    p = p*p*p*p;
    p = p*p;
    ret = (p.x+p.y);
    ret = pow(ret, 1./8.);
    return ret;
}

//--------- UTILITIES's --------------
void rotate(inout vec2 v, float angle)
{
    v = vec2(cos(angle)*v.x+sin(angle)*v.y,-sin(angle)*v.x+cos(angle)*v.y);
}

vec2 rot(vec2 p, vec2 ang) 
{
    float   c = cos(ang.x);
    float   s = sin(ang.y);
    mat2    m = mat2(c, -s, s, c);
    return (p * m);
}

vec3 camera(vec2 uv) 
{
    float fov  = 1.;
    vec3 forw  = vec3(0.0, 0.0, -1.0);
    vec3 right = vec3(1.0, 0.0, 0.0);
    vec3 up    = vec3(0.0, 1.0, 0.0);
    return (normalize((uv.x) * right + (uv.y) * up + fov * forw));
}

vec3 calcNormal( in vec3 pos, float e, vec3 dir) 
{
    vec3 eps = vec3(e,0.0,0.0);
    return normalize(vec3(
           march(pos+eps.xyy, dir).y - march(pos-eps.xyy, dir).y,
           march(pos+eps.yxy, dir).y - march(pos-eps.yxy, dir).y,
           march(pos+eps.yyx, dir).y - march(pos-eps.yyx, dir).y ));
}