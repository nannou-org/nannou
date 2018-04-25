#include "noise3D.glsl"
#include "noise2D.glsl"

float fbm(in vec2 v, int octaves)
{
	float res = 0.0;
	float scale = 1.0;
	for(int i=0; i<8; i++)
	{
		if(i >= octaves) break;
		res += snoise(v) * scale;
		v *= vec2(2.0, 2.0);
		scale *= 0.5;
	}
	return res;
}

float fbm(in vec3 v, int octaves)
{
	float res = 0.0;
	float scale = 1.0;
	for(int i=0; i<8; i++)
	{
		if(i >= octaves) break;
		res += snoise(v) * scale;
		v *= vec3(2.0, 2.0, 2.0);
		scale *= 0.5;
	}
	return res;
}


float fbm_abs(in vec2 v, int octaves)
{
	float res = 0.0;
	float scale = 1.0;
	for(int i=0; i<8; i++)
	{
		if(i >= octaves) break;
		res += abs(snoise(v)) * scale;
		v *= vec2(2.0, 2.0);
		scale *= 0.5;
	}
	return res;
}

float fbm_abs(in vec3 v, int octaves)
{
	float res = 0.0;
	float scale = 1.0;
	for(int i=0; i<8; i++)
	{
		if(i >= octaves) break;
		res += abs(snoise(v)) * scale;
		v *= vec3(2.0, 2.0, 2.0);
		scale *= 0.5;
	}
	return res;
}
