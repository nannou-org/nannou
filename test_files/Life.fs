/*{
	"DESCRIPTION": "Based on Conway Game of Life",
	"CREDIT": "VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Noise"
	],
	"INPUTS": [
		{
			"NAME": "restartNow",
			"TYPE": "event"
		},
		{
			"NAME": "startThresh",
			"TYPE": "float",
			"DEFAULT": 0.5,
			"MIN": 0.0,
			"MAX": 1.0
		},
		{
			"NAME": "randomRegrowth",
			"TYPE": "float",
			"DEFAULT": 0.0,
			"MIN": 0.0,
			"MAX": 0.1
		},
		{
			"NAME": "randomDeath",
			"TYPE": "float",
			"DEFAULT": 0.0,
			"MIN": 0.0,
			"MAX": 0.1
		}
	],
	"PASSES": [
		{
			"TARGET":"lastData",
			"PERSISTENT": true
		}
	]
	
}*/


/*

Any live cell with fewer than two live neighbours dies, as if caused by under-population.
Any live cell with two or three live neighbours lives on to the next generation.
Any live cell with more than three live neighbours dies, as if by over-population.
Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.

*/



#if __VERSION__ <= 120
varying vec2 left_coord;
varying vec2 right_coord;
varying vec2 above_coord;
varying vec2 below_coord;

varying vec2 lefta_coord;
varying vec2 righta_coord;
varying vec2 leftb_coord;
varying vec2 rightb_coord;
#else
in vec2 left_coord;
in vec2 right_coord;
in vec2 above_coord;
in vec2 below_coord;

in vec2 lefta_coord;
in vec2 righta_coord;
in vec2 leftb_coord;
in vec2 rightb_coord;
#endif



float gray(vec4 n)
{
	return (n.r + n.g + n.b)/3.0;
}

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}


void main()	{
	vec4		inputPixelColor = vec4(0.0);
	vec2		loc = gl_FragCoord.xy;
	
	if ((TIME < 0.1)||(restartNow))	{
		//	randomize the start conditions
		float	alive = rand(vec2(TIME+1.0,2.1*TIME+0.1)*loc);
		if (alive > 1.0 - startThresh)	{
			inputPixelColor = vec4(1.0);
		}
	}
	else	{
		vec4	color = IMG_PIXEL(lastData, loc);
		vec4	colorL = IMG_PIXEL(lastData, left_coord);
		vec4	colorR = IMG_PIXEL(lastData, right_coord);
		vec4	colorA = IMG_PIXEL(lastData, above_coord);
		vec4	colorB = IMG_PIXEL(lastData, below_coord);

		vec4	colorLA = IMG_PIXEL(lastData, lefta_coord);
		vec4	colorRA = IMG_PIXEL(lastData, righta_coord);
		vec4	colorLB = IMG_PIXEL(lastData, leftb_coord);
		vec4	colorRB = IMG_PIXEL(lastData, rightb_coord);
		
		float	neighborSum = gray(colorL + colorR + colorA + colorB + colorLA + colorRA + colorLB + colorRB);
		float	state = gray(color);
		
		//	live cell
		if (state > 0.0)	{
			if (neighborSum < 2.0)	{
				//	under population
				inputPixelColor = vec4(0.0);
			}
			else if (neighborSum < 4.0)	{
				//	status quo
				inputPixelColor = vec4(1.0);
				
				//	spontaneous death?
				float	alive = rand(vec2(TIME+1.0,2.1*TIME+0.1)*loc);
				if (alive > 1.0 - randomDeath)	{
					inputPixelColor = vec4(0.0);
				}
			}
			else	{
				//	over population
				inputPixelColor = vec4(0.0);
			}
		}
		//	dead cell
		else	{
			if ((neighborSum > 2.0)&&(neighborSum < 4.0))	{
				//	reproduction
				inputPixelColor = vec4(1.0);
			}
			else if (neighborSum < 2.0)	{
				//	spontaneous reproduction
				float	alive = rand(vec2(TIME+1.0,2.1*TIME+0.1)*loc);
				if (alive > 1.0 - randomRegrowth)	{
					inputPixelColor = vec4(1.0);
				}
			}
		}
	}
	
	gl_FragColor = inputPixelColor;
}
