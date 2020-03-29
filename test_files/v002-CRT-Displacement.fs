/*{
    "CATEGORIES": [
        "Distortion Effect",
        "Retro",
        "v002"
    ],
    "CREDIT": "by vade",
    "DESCRIPTION": "CRT Displacement, emulating the look of curved CRT Displays",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "Amount",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

void main (void) 
{ 
	vec2 t1, t2;
	vec2 ctr = RENDERSIZE / 2.0;
	
	t1 = gl_FragCoord.xy;
	
	float a = -0.0;
	float b = -.1 * Amount;
	float c = -.0;
	float d = 1.0  - 1.1 * ( a + b + c );
	float r1, r2;
	float unit = length(ctr) / 2.0;
	
	r1 = distance( t1, ctr )/unit;
	r2 =  r1 *( r1*( r1 * (a*r1 + b) + c) + d );
	float sc = step( 0.0 , r1) * ( r1/(r2 + .000001)) + (1.0 - step( 0.0 , r1));

	t2  = ctr + ( t1 - ctr) * sc;
	
	gl_FragColor = IMG_PIXEL(inputImage, t2);
			
	if ((t2.x < 0.0)
		||(t2.y < 0.0)
		||(t2.x > RENDERSIZE.x)
		||(t2.y > RENDERSIZE.y)) 
	{
		gl_FragColor = vec4(0.0);
	}
}