float		PI_CONST = 3.14159265359;


//	with help from http://duriansoftware.com/joe/An-intro-to-modern-OpenGL.-Chapter-3:-3D-transformation-and-projection.html
//	and http://en.wikipedia.org/wiki/Rotation_matrix


mat4 view_frustum(
    float angle_of_view,
    float aspect_ratio,
    float z_near,
    float z_far
) {
    return mat4(
        vec4(1.0/tan(angle_of_view),           0.0, 0.0, 0.0),
        vec4(0.0, aspect_ratio/tan(angle_of_view),  0.0, 0.0),
        vec4(0.0, 0.0,    (z_far+z_near)/(z_far-z_near), 1.0),
        vec4(0.0, 0.0, -2.0*z_far*z_near/(z_far-z_near), 0.0)
    );
}

mat4 scale(float x, float y, float z)
{
    return mat4(
        vec4(x,   0.0, 0.0, 0.0),
        vec4(0.0, y,   0.0, 0.0),
        vec4(0.0, 0.0, z,   0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );
}

mat4 translate(float x, float y, float z)
{
    return mat4(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(x,   y,   z,   1.0)
    );
}

mat4 rotate_x(float theta)
{
    return mat4(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, cos(theta*PI_CONST), sin(theta*PI_CONST), 0.0),
        vec4(0.0, -sin(theta*PI_CONST), cos(theta*PI_CONST), 0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );
}

mat4 rotate_y(float theta)
{
    return mat4(
        vec4(cos(theta*PI_CONST), 0.0, sin(theta*PI_CONST), 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(-sin(theta*PI_CONST), 0, cos(theta*PI_CONST), 0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );
}

mat4 rotate_z(float theta)
{
    return mat4(
        vec4(cos(theta*PI_CONST), -sin(theta*PI_CONST), 0.0, 0.0),
        vec4(sin(theta*PI_CONST), cos(theta*PI_CONST), 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );
}

void main()
{
	isf_vertShaderInit();
	
	vec4 position = gl_Position;

    gl_Position = view_frustum(radians(45.0), RENDERSIZE.x/RENDERSIZE.y, 0.0, 2.0)
        * translate(0.0, 0.0, RENDERSIZE.x/RENDERSIZE.y)
        * rotate_x(-xrot)
        * rotate_y(yrot)
        * rotate_z(-zrot)
        * scale(zoom*RENDERSIZE.x/RENDERSIZE.y, zoom, zoom)
        * position;

}
