
#if __VERSION__ <= 120
varying mat2 rotmat;
#else
out mat2 rotmat;
#endif

void main()
{
	isf_vertShaderInit();

	// setup basic rotation matrix here
	float theta = radians(angle); 

	// dont need to compute this more than once..
	float c = cos(theta);
	float s = sin(theta);

	// rotation matrix
	rotmat = mat2(c,s,-s,c);
}