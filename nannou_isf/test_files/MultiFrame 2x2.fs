/*{
	"DESCRIPTION": "buffers the last 3 frames and draws a 2x2 grid of the 4 current frames available",
	"ISFVSN": "2",
	"CREDIT": "by VIDVOX",
	"CATEGORIES": [
		"Tile Effect", "Stylize"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "lag",
			"TYPE": "bool",
			"DEFAULT": 1.0
		},
		{
			"NAME": "hueShift",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		}
	],
	"PASSES": [
		{
			"TARGET":"buffer3",
			"PERSISTENT": true,
			"WIDTH": "$WIDTH/2.0",
			"HEIGHT": "$HEIGHT/2.0"
		},
		{
			"TARGET":"buffer2",
			"PERSISTENT": true,
			"WIDTH": "$WIDTH/2.0",
			"HEIGHT": "$HEIGHT/2.0"
		},
		{
			"TARGET":"buffer1",
			"PERSISTENT": true,
			"WIDTH": "$WIDTH/2.0",
			"HEIGHT": "$HEIGHT/2.0"
		},
		{
		
		}
	]
	
}*/



vec3 rgb2hsv(vec3 c)	{
	vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	//vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
	//vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
	vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
	vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);
	
	float d = q.x - min(q.w, q.y);
	float e = 1.0e-10;
	return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)	{
	vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
	vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
	return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}



void main()
{
	//	first pass: read the "buffer2" into "buffer3"
	//	apply lag on each pass
	if (PASSINDEX == 0)	{
		if ((lag == false)||(mod(floor(TIME*60.0+2.0),6.0)<=1.0))	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer2);
		}
		else	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer3);
		}
	}
	//	first pass: read the "buffer1" into "buffer2"
	else if (PASSINDEX == 1)	{
		if ((lag == false)||(mod(floor(TIME*60.0+2.0),6.0)<=1.0))	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer1);
		}
		else	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer2);
		}
	}
	//	third pass: read from "inputImage" into "buffer1"
	else if (PASSINDEX == 2)	{
		if ((lag == false)||(mod(floor(TIME*60.0+2.0),6.0)<=1.0))	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(inputImage);
		}
		else	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer1);
		}
	}
	else if (PASSINDEX == 3)	{
		//	Figure out which section I'm in and draw the appropriate buffer there
		vec2 tex = isf_FragNormCoord;
		vec4 color = vec4(0.0);
		//	TL – buffer1
		if ((tex.x < 0.5) && (tex.y > 0.5))	{
			tex.x = tex.x * 2.0;
			tex.y = (tex.y - 0.5) * 2.0;
			color = IMG_NORM_PIXEL(inputImage, tex);
		}
		//	TR – buffer2
		else if ((tex.x > 0.5) && (tex.y > 0.5))	{
			tex.x = (tex.x - 0.5) * 2.0;
			tex.y = (tex.y - 0.5) * 2.0;
			
			color = IMG_NORM_PIXEL(buffer1, tex);
			color.rgb = rgb2hsv(color.rgb);
			color.r = mod(color.r + hueShift, 1.0);
			color.rgb = hsv2rgb(color.rgb);
		}
		//	BR – buffer2
		else if ((tex.x < 0.5) && (tex.y < 0.5))	{
			tex = tex * 2.0;
			
			color = IMG_NORM_PIXEL(buffer2, tex);
			color.rgb = rgb2hsv(color.rgb);
			color.r = mod(color.r + 2.0 * hueShift, 1.0);
			color.rgb = hsv2rgb(color.rgb);	
		}
		else	{
			tex.x = (tex.x - 0.5) * 2.0;
			tex.y = tex.y * 2.0;
			
			color = IMG_NORM_PIXEL(buffer3, tex);
			color.rgb = rgb2hsv(color.rgb);
			color.r = mod(color.r + 3.0 * hueShift, 1.0);
			color.rgb = hsv2rgb(color.rgb);		
		}
		//	BL - buffer3
		gl_FragColor = color;
	}
}
