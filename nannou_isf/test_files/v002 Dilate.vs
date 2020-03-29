
#if __VERSION__ <= 120
varying vec2 texcoord0;
varying vec2 texcoord1;
varying vec2 texcoord2;
varying vec2 texcoord3;
varying vec2 texcoord4;
varying vec2 texcoord5;
varying vec2 texcoord6;
varying vec2 texcoord7;
#else
out vec2 texcoord0;
out vec2 texcoord1;
out vec2 texcoord2;
out vec2 texcoord3;
out vec2 texcoord4;
out vec2 texcoord5;
out vec2 texcoord6;
out vec2 texcoord7;
#endif

void main()
{
	isf_vertShaderInit();
	// perform standard transform on vertex
	//gl_Position = ftransform();

	// transform texcoord        
	vec2 texcoord = vec2(isf_FragNormCoord[0],isf_FragNormCoord[1]);

	// get sample positions
	texcoord0 = texcoord + vec2(-amount, -amount);
	texcoord1 = texcoord + vec2( 0,      -amount);
	texcoord2 = texcoord + vec2( amount, -amount);
	texcoord3 = texcoord + vec2(-amount,  0);
	texcoord4 = texcoord + vec2( amount,  0);
	texcoord5 = texcoord + vec2(-amount,  amount);
	texcoord6 = texcoord + vec2( 0,       amount);
	texcoord7 = texcoord + vec2( amount,  amount);
}
