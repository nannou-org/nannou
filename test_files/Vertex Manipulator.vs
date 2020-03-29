

void main()
{
	isf_vertShaderInit();
	
	vec4 position = gl_Position;
	vec4 coord;
	if ((position.x<0.0)&&(position.y<0.0))	{
	    coord.xy = bottomleft / RENDERSIZE;
	    coord.x = coord.x * 2.0;
	    coord.y = coord.y * 2.0;
	}
	else if ((position.x>0.0)&&(position.y<0.0))	{
	    coord.xy = bottomright / RENDERSIZE;
	    coord.x = (1.0-coord.x) * -2.0;
	    coord.y = coord.y * 2.0;
	}
	else if (position.x<0.0)	{
		coord.xy = topleft / RENDERSIZE;
	    coord.x = coord.x * 2.0;
	    coord.y = (1.0-coord.y) * -2.0;
	}
	else	{
		coord.xy = topright / RENDERSIZE;
	    coord.x = (1.0-coord.x) * -2.0;
	    coord.y = (1.0-coord.y) * -2.0;
	}
	gl_Position.xy = position.xy + coord.xy;

}
