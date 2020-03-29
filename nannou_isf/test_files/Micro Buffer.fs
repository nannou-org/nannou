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
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 9.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "inputDelay2",
			"LABEL": "Buffer 2",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 9.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "inputDelay3",
			"LABEL": "Buffer 3",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 9.0,
			"DEFAULT": 0.0
		},
		{
			"NAME": "inputRate",
			"LABEL": "Buffer Lag",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 20.0,
			"DEFAULT": 1.0
		},
		{
			"NAME": "mode",
			"VALUES": [
				0,
				1,
				2
			],
			"LABELS": [
				"Single",
				"Double",
				"Triple"
			],
			"DEFAULT": 0,
			"TYPE": "long"
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
		vec4 returnMe = vec4(0.0);
		vec4 color = vec4(0.0);
		float pixelBuffer = inputDelay;
		
		//	If the glitch factor is turned on I might randomly go to another delay point

		if (pixelBuffer < 1.0)	{
			color = IMG_NORM_PIXEL(inputImage, tex);
		}
		else if (pixelBuffer < 2.0)	{
			color = IMG_NORM_PIXEL(buffer1, tex);
		}
		else if (pixelBuffer < 3.0)	{
			color = IMG_NORM_PIXEL(buffer2, tex);
		}
		else if (pixelBuffer < 4.0)	{
			color = IMG_NORM_PIXEL(buffer3, tex);
		}
		else if (pixelBuffer < 5.0)	{
			color = IMG_NORM_PIXEL(buffer4, tex);
		}
		else if (pixelBuffer < 6.0)	{
			color = IMG_NORM_PIXEL(buffer5, tex);
		}
		else if (pixelBuffer < 7.0)	{
			color = IMG_NORM_PIXEL(buffer6, tex);
		}
		else if (pixelBuffer < 8.0)	{
			color = IMG_NORM_PIXEL(buffer7, tex);
		}
		else	{
			color = IMG_NORM_PIXEL(buffer8, tex);
		}
		
		returnMe = color;
		
		if ((mode == 1)||(mode == 2))	{
			pixelBuffer = inputDelay2;
			if (pixelBuffer < 1.0)	{
				color = IMG_NORM_PIXEL(inputImage, tex);
			}
			else if (pixelBuffer < 2.0)	{
				color = IMG_NORM_PIXEL(buffer1, tex);
			}
			else if (pixelBuffer < 3.0)	{
				color = IMG_NORM_PIXEL(buffer2, tex);
			}
			else if (pixelBuffer < 4.0)	{
				color = IMG_NORM_PIXEL(buffer3, tex);
			}
			else if (pixelBuffer < 5.0)	{
				color = IMG_NORM_PIXEL(buffer4, tex);
			}
			else if (pixelBuffer < 6.0)	{
				color = IMG_NORM_PIXEL(buffer5, tex);
			}
			else if (pixelBuffer < 7.0)	{
				color = IMG_NORM_PIXEL(buffer6, tex);
			}
			else if (pixelBuffer < 8.0)	{
				color = IMG_NORM_PIXEL(buffer7, tex);
			}
			else	{
				color = IMG_NORM_PIXEL(buffer8, tex);
			}
			returnMe = (returnMe + color);
		}
		
		if (mode == 2)	{
			pixelBuffer = inputDelay3;
			if (pixelBuffer < 1.0)	{
				color = IMG_NORM_PIXEL(inputImage, tex);
			}
			else if (pixelBuffer < 2.0)	{
				color = IMG_NORM_PIXEL(buffer1, tex);
			}
			else if (pixelBuffer < 3.0)	{
				color = IMG_NORM_PIXEL(buffer2, tex);
			}
			else if (pixelBuffer < 4.0)	{
				color = IMG_NORM_PIXEL(buffer3, tex);
			}
			else if (pixelBuffer < 5.0)	{
				color = IMG_NORM_PIXEL(buffer4, tex);
			}
			else if (pixelBuffer < 6.0)	{
				color = IMG_NORM_PIXEL(buffer5, tex);
			}
			else if (pixelBuffer < 7.0)	{
				color = IMG_NORM_PIXEL(buffer6, tex);
			}
			else if (pixelBuffer < 8.0)	{
				color = IMG_NORM_PIXEL(buffer7, tex);
			}
			else	{
				color = IMG_NORM_PIXEL(buffer8, tex);
			}
			returnMe = (returnMe + color);
		}
		
		returnMe = returnMe / (1.0+float(mode));

		gl_FragColor = returnMe;
	}
}
