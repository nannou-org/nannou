/*{
	"DESCRIPTION": "Buffers 8 recent frames",
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Glitch"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "inputDelay",
			"LABEL": "Buffer",
			"TYPE": "color",
			"DEFAULT": [
				0.25,
				0.5,
				0.75,
				0.5
			]
		},
		{
			"NAME": "inputRate",
			"LABEL": "Buffer Lag",
			"TYPE": "float",
			"MIN": 1.0,
			"MAX": 20.0,
			"DEFAULT": 4.0
		},
		{
			"NAME": "glitch_size",
			"LABEL": "Size",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 0.5,
			"DEFAULT": 0.1
		},
		{
			"NAME": "glitch_horizontal",
			"LABEL": "Horizontal Amount",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.2
		},
		{
			"NAME": "glitch_vertical",
			"LABEL": "Vertical Amount",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "randomize_size",
			"LABEL": "Randomize Size",
			"TYPE": "bool",
			"DEFAULT": 1.0
		},
		{
			"NAME": "randomize_position",
			"LABEL": "Randomize Position",
			"TYPE": "bool",
			"DEFAULT": 0.0
		},
		{
			"NAME": "randomize_zoom",
			"LABEL": "Randomize Zoom",
			"TYPE": "bool",
			"DEFAULT": 0.0
		}
	],
	"PASSES": [
		{
			"TARGET":"lastRow",
			"WIDTH": "1",
			"HEIGHT": "1",
			"PERSISTENT": true,
			"DESCRIPTION": "this buffer stores the last frame's odd / even state"
		},
		{
			"TARGET":"buffer8",
			"PERSISTENT": true
		},
		{
			"TARGET":"buffer7",
			"PERSISTENT": true
		},
		{
			"TARGET":"buffer6",
			"PERSISTENT": true
		},
		{
			"TARGET":"buffer5",
			"PERSISTENT": true
		},
		{
			"TARGET":"buffer4",
			"PERSISTENT": true
		},
		{
			"TARGET":"buffer3",
			"PERSISTENT": true
		},
		{
			"TARGET":"buffer2",
			"PERSISTENT": true
		},
		{
			"TARGET":"buffer1",
			"PERSISTENT": true
		},
		{
		
		}
	]
	
}*/


float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}


void main()
{
	//	first pass: read the "buffer7" into "buffer8"
	//	apply lag on each pass
	//	if this is the first pass, i'm going to read the position from the "lastRow" image, and write a new position based on this and the hold variables
	if (PASSINDEX == 0)	{
		vec4		srcPixel = IMG_PIXEL(lastRow,vec2(0.5));
		//	i'm only using the X and Y components, which are the X and Y offset (normalized) for the frame
		if (inputRate == 0.0)	{
			srcPixel.x = 0.0;
			srcPixel.y = 0.0;
		}
		else if (inputRate <= 1.0)	{
			srcPixel.x = (srcPixel.x) > 0.5 ? 0.0 : 1.0;
			srcPixel.y = 0.0;
		}
		else {
			srcPixel.x = srcPixel.x + 1.0 / inputRate + srcPixel.y;
			if (srcPixel.x > 1.0)	{
				srcPixel.y = mod(srcPixel.x, 1.0);
				srcPixel.x = 0.0;
			}
		}
		gl_FragColor = srcPixel;
	}
	if (PASSINDEX == 1)	{
		vec4		lastRow = IMG_PIXEL(lastRow,vec2(0.5));
		if (lastRow.x == 0.0)	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer7);
		}
		else	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer8);
		}
	}
	else if (PASSINDEX == 2)	{
		vec4		lastRow = IMG_PIXEL(lastRow,vec2(0.5));
		if (lastRow.x == 0.0)	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer6);
		}
		else	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer7);
		}
	}
	else if (PASSINDEX == 3)	{
		vec4		lastRow = IMG_PIXEL(lastRow,vec2(0.5));
		if (lastRow.x == 0.0)	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer5);
		}
		else	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer6);
		}
	}
	else if (PASSINDEX == 4)	{
		vec4		lastRow = IMG_PIXEL(lastRow,vec2(0.5));
		if (lastRow.x == 0.0)	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer4);
		}
		else	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer5);
		}
	}
	else if (PASSINDEX == 5)	{
		vec4		lastRow = IMG_PIXEL(lastRow,vec2(0.5));
		if (lastRow.x == 0.0)	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer3);
		}
		else	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer4);
		}
	}
	else if (PASSINDEX == 6)	{
		vec4		lastRow = IMG_PIXEL(lastRow,vec2(0.5));
		if (lastRow.x == 0.0)	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer2);
		}
		else	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer3);
		}
	}
	else if (PASSINDEX == 7)	{
		vec4		lastRow = IMG_PIXEL(lastRow,vec2(0.5));
		if (lastRow.x == 0.0)	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer1);
		}
		else	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer2);
		}
	}
	else if (PASSINDEX == 8)	{
		vec4		lastRow = IMG_PIXEL(lastRow,vec2(0.5));
		if (lastRow.x == 0.0)	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(inputImage);
		}
		else	{
			gl_FragColor = IMG_THIS_NORM_PIXEL(buffer1);
		}
	}
	else if (PASSINDEX == 9)	{
		//	Figure out which section I'm in and draw the appropriate buffer there
		vec2 tex = isf_FragNormCoord;
		vec4 color = vec4(0.0);		
		//	figure out the "input delay shift" for this pixel...
		float randomDelayShift = 0.0;
		
		vec2 xy; 
		xy.x = isf_FragNormCoord[0];
		xy.y = isf_FragNormCoord[1];
	
		//	quantize the xy to the glitch_amount size
		//xy = floor(xy / glitch_size) * glitch_size;
		vec2 random;

		float local_glitch_size = glitch_size;
		float random_offset = 0.0;
	
		if (randomize_size)	{
			random_offset = mod(rand(vec2(TIME,TIME)), 1.0);
			local_glitch_size = random_offset * glitch_size;
		}
	
		if (local_glitch_size > 0.0)	{
			random.x = rand(vec2(floor(random_offset + xy.y / local_glitch_size) * local_glitch_size, TIME));
			random.y = rand(vec2(floor(random_offset + xy.x / local_glitch_size) * local_glitch_size, TIME));
		}
		else	{
			random.x = rand(vec2(xy.x, TIME));
			random.y = rand(vec2(xy.y, TIME));
		}
	
		//	if doing a horizontal glitch do a random shift
		if ((random.x < glitch_horizontal)&&(random.y < glitch_vertical))	{
			randomDelayShift = clamp(random.x + random.y, 0.0, 2.0);
		}
		else if (random.x < glitch_horizontal)	{
			randomDelayShift = clamp(random.x + random.y, 0.0, 2.0);
		}
		else if (random.y < glitch_vertical)	{
			randomDelayShift = clamp(random.x + random.y, 0.0, 2.0);
		}
		
		vec4 pixelBuffer = randomDelayShift * inputDelay * 9.0;
		
		if (randomize_zoom)	{
			if ((random.x < glitch_horizontal)&&(random.y < glitch_vertical))	{
				float level = (random.x + random.y) / 3.0 + 0.90;
				tex = (tex - vec2(0.5))*(1.0/level) + vec2(0.5);
			}
			else if (random.x < glitch_horizontal)	{
				float level = (random.x) / 2.0 + 0.95;
				tex = (tex - vec2(0.5))*(1.0/level) + vec2(0.5);
			}
			else if (random.y < glitch_vertical)	{
				float level = (random.y) / 2.0 + 0.95;
				tex = (tex - vec2(0.5))*(1.0/level) + vec2(0.5);
			}
		}
		
		if (randomize_position)	{
			if ((random.x < glitch_horizontal)&&(random.y < glitch_vertical))	{
				tex.x = mod(tex.x + inputDelay.r * random.x, 1.0);
				tex.y = mod(tex.y + inputDelay.r * random.y, 1.0);
			}
			else if (random.x < glitch_horizontal)	{
				tex.y = mod(tex.y + inputDelay.r * random.x, 1.0);
			}
			else if (random.y < glitch_vertical)	{
				tex.x = mod(tex.x + inputDelay.r * random.y, 1.0);
			}
			//	apply small random zoom too
		}
		
		if (pixelBuffer.r < 1.0)	{
			color.r = IMG_NORM_PIXEL(inputImage, tex).r;
		}
		else if (pixelBuffer.r < 2.0)	{
			color.r = IMG_NORM_PIXEL(buffer1, tex).r;
		}
		else if (pixelBuffer.r < 3.0)	{
			color.r = IMG_NORM_PIXEL(buffer2, tex).r;
		}
		else if (pixelBuffer.r < 4.0)	{
			color.r = IMG_NORM_PIXEL(buffer3, tex).r;
		}
		else if (pixelBuffer.r < 5.0)	{
			color.r = IMG_NORM_PIXEL(buffer4, tex).r;
		}
		else if (pixelBuffer.r < 6.0)	{
			color.r = IMG_NORM_PIXEL(buffer5, tex).r;
		}
		else if (pixelBuffer.r < 7.0)	{
			color.r = IMG_NORM_PIXEL(buffer6, tex).r;
		}
		else if (pixelBuffer.r < 8.0)	{
			color.r = IMG_NORM_PIXEL(buffer7, tex).r;
		}
		else	{
			color.r = IMG_NORM_PIXEL(buffer8, tex).r;
		}
		
		if (randomize_position)	{
			if ((random.x < glitch_horizontal)&&(random.y < glitch_vertical))	{
				tex.x = mod(tex.x + random.x * inputDelay.g, 1.0);
				tex.y = mod(tex.y + random.y * inputDelay.g, 1.0);
			}
			else if (random.x < glitch_horizontal)	{
				tex.y = mod(tex.y + random.x * inputDelay.g, 1.0);
			}
			else if (random.y < glitch_vertical)	{
				tex.x = mod(tex.x + random.y * inputDelay.g, 1.0);
			}
			//	apply small random zoom too
			//float level = inputDelay.g * random.x / 5.0 + 0.9;
			//tex = (tex - vec2(0.5))*(1.0/level) + vec2(0.5);
		}
		
		if (pixelBuffer.g < 1.0)	{
			color.g = IMG_NORM_PIXEL(inputImage, tex).g;
		}
		else if (pixelBuffer.g < 2.0)	{
			color.g = IMG_NORM_PIXEL(buffer1, tex).g;
		}
		else if (pixelBuffer.g < 3.0)	{
			color.g = IMG_NORM_PIXEL(buffer2, tex).g;
		}
		else if (pixelBuffer.g < 4.0)	{
			color.g = IMG_NORM_PIXEL(buffer3, tex).g;
		}
		else if (pixelBuffer.g < 5.0)	{
			color.g = IMG_NORM_PIXEL(buffer4, tex).g;
		}
		else if (pixelBuffer.g < 6.0)	{
			color.g = IMG_NORM_PIXEL(buffer5, tex).g;
		}
		else if (pixelBuffer.g < 7.0)	{
			color.g = IMG_NORM_PIXEL(buffer6, tex).g;
		}
		else if (pixelBuffer.g < 8.0)	{
			color.g = IMG_NORM_PIXEL(buffer7, tex).g;
		}
		else	{
			color.g = IMG_NORM_PIXEL(buffer8, tex).g;
		}
		
		if (randomize_position)	{
			if ((random.x < glitch_horizontal)&&(random.y < glitch_vertical))	{
				tex.x = mod(tex.x + random.x * inputDelay.b, 1.0);
				tex.y = mod(tex.y + random.y * inputDelay.b, 1.0);
			}
			else if (random.x < glitch_horizontal)	{
				tex.y = mod(tex.y + random.x * inputDelay.b, 1.0);
			}
			else if (random.y < glitch_vertical)	{
				tex.x = mod(tex.x + random.y * inputDelay.b, 1.0);
			}
			//	apply small random zoom too
			//float level = inputDelay.b * random.x / 5.0 + 0.9;
			//tex = (tex - vec2(0.5))*(1.0/level) + vec2(0.5);
		}
		
		if (pixelBuffer.b < 1.0)	{
			color.b = IMG_NORM_PIXEL(inputImage, tex).b;
		}
		else if (pixelBuffer.b < 2.0)	{
			color.b = IMG_NORM_PIXEL(buffer1, tex).b;
		}
		else if (pixelBuffer.b < 3.0)	{
			color.b = IMG_NORM_PIXEL(buffer2, tex).b;
		}
		else if (pixelBuffer.b < 4.0)	{
			color.b = IMG_NORM_PIXEL(buffer3, tex).b;
		}
		else if (pixelBuffer.b < 5.0)	{
			color.b = IMG_NORM_PIXEL(buffer4, tex).b;
		}
		else if (pixelBuffer.b < 6.0)	{
			color.b = IMG_NORM_PIXEL(buffer5, tex).b;
		}
		else if (pixelBuffer.b < 7.0)	{
			color.b = IMG_NORM_PIXEL(buffer6, tex).b;
		}
		else if (pixelBuffer.b < 8.0)	{
			color.b = IMG_NORM_PIXEL(buffer7, tex).b;
		}
		else	{
			color.b = IMG_NORM_PIXEL(buffer8, tex).b;
		}
		
		if (randomize_position)	{
			if ((random.x < glitch_horizontal)&&(random.y < glitch_vertical))	{
				tex.x = mod(tex.x + random.x * inputDelay.a, 1.0);
				tex.y = mod(tex.y + random.y * inputDelay.a, 1.0);
			}
			else if (random.x < glitch_horizontal)	{
				tex.y = mod(tex.y + random.x * inputDelay.a, 1.0);
			}
			else if (random.y < glitch_vertical)	{
				tex.x = mod(tex.x + random.y * inputDelay.a, 1.0);
			}
			//	apply small random zoom too
			//float level = inputDelay.a * random.x / 5.0 + 0.9;
			//tex = (tex - vec2(0.5))*(1.0/level) + vec2(0.5);
		}
		
		if (pixelBuffer.a < 1.0)	{
			color.a = IMG_NORM_PIXEL(inputImage, tex).a;
		}
		else if (pixelBuffer.a < 2.0)	{
			color.a = IMG_NORM_PIXEL(buffer1, tex).a;
		}
		else if (pixelBuffer.a < 3.0)	{
			color.a = IMG_NORM_PIXEL(buffer2, tex).a;
		}
		else if (pixelBuffer.a < 4.0)	{
			color.a = IMG_NORM_PIXEL(buffer3, tex).a;
		}
		else if (pixelBuffer.a < 5.0)	{
			color.a = IMG_NORM_PIXEL(buffer4, tex).a;
		}
		else if (pixelBuffer.a < 6.0)	{
			color.a = IMG_NORM_PIXEL(buffer5, tex).a;
		}
		else if (pixelBuffer.a < 7.0)	{
			color.a = IMG_NORM_PIXEL(buffer6, tex).a;
		}
		else if (pixelBuffer.a < 8.0)	{
			color.a = IMG_NORM_PIXEL(buffer7, tex).a;
		}
		else	{
			color.a = IMG_NORM_PIXEL(buffer8, tex).a;
		}

		gl_FragColor = color;
	}
}
