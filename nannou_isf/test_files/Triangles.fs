
/*{
	"CREDIT": "by VIDVOX",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Stylize"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "cell_size",
			"TYPE": "float",
			"MIN": 0.001,
			"MAX": 0.5,
			"DEFAULT": 0.125
		},
		{
			"NAME": "style",
			"VALUES": [
				0,
				1
			],
			"LABELS": [
				"Squared",
				"Diamond"
			],
			"DEFAULT": 0,
			"TYPE": "long"
		}
	]
}*/

#ifndef GL_ES
float distance (vec2 center, vec2 pt)
{
	float tmp = pow(center.x-pt.x,2.0)+pow(center.y-pt.y,2.0);
	return pow(tmp,0.5);
}
#endif

void main()
{
// CALCULATE EDGES OF CURRENT CELL
	//	At 0.0 just do a pass-thru
	if (cell_size == 0.0)	{
		gl_FragColor = IMG_THIS_PIXEL(inputImage);
	}
	else	{
		// Position of current pixel
		vec2 xy; 
		xy.x = isf_FragNormCoord[0];
		xy.y = isf_FragNormCoord[1];


		// Left and right of tile
		float CellWidth = cell_size;
		float CellHeight = cell_size;
		
		CellHeight = cell_size * RENDERSIZE.x / RENDERSIZE.y;

		float x1 = floor(xy.x / CellWidth)*CellWidth;
		float x2 = clamp((ceil(xy.x / CellWidth)*CellWidth), 0.0, 1.0);
		// Top and bottom of tile
		float y1 = floor(xy.y / CellHeight)*CellHeight;
		float y2 = clamp((ceil(xy.y / CellHeight)*CellHeight), 0.0, 1.0);
		
		//	get the normalized local coords in the cell
		float x = (xy.x-x1) / CellWidth;
		float y = (xy.y-y1) / CellHeight;
		vec4 avgClr = vec4(0.0);
		
		//	style 0, two right triangles making a square
		if (style == 0)	{
			//	if above the center line...
			if (x < y)	{
				// Average bottom left, top left, center and top right pixels
				vec4 avgL = (IMG_NORM_PIXEL(inputImage, vec2(x1, y1))+(IMG_NORM_PIXEL(inputImage, vec2(x1, y2)))) / 2.0;
				vec4 avgR = IMG_NORM_PIXEL(inputImage, vec2(x2, y2));
				vec4 avgC = IMG_NORM_PIXEL(inputImage, vec2(x1+(CellWidth/2.0), y2+(CellHeight/2.0)));	// Average the averages + centre
				avgClr = (avgL+avgR+avgC) / 3.0;
			}
			else	{
				// Average bottom right, bottom left, center and top right pixels
				vec4 avgR = (IMG_NORM_PIXEL(inputImage, vec2(x2, y1))+(IMG_NORM_PIXEL(inputImage, vec2(x2, y2)))) / 2.0;
				vec4 avgL = IMG_NORM_PIXEL(inputImage, vec2(x1, y1));
				vec4 avgC = IMG_NORM_PIXEL(inputImage, vec2(x1+(CellWidth/2.0), y2+(CellHeight/2.0)));	// Average the averages + centre
				avgClr = (avgL+avgR+avgC) / 3.0;
			}
		}
		//	style 1, four triangles making a square
		else {
			//	if above the B2T center line and below the T2B center line...
			if ((x > y)&&(x < 1.0 - y))	{
				// Average bottom left, bottom right, center
				vec4 avgL = IMG_NORM_PIXEL(inputImage, vec2(x1, y1));
				vec4 avgR = IMG_NORM_PIXEL(inputImage, vec2(x2, y1));
				vec4 avgC = IMG_NORM_PIXEL(inputImage, vec2(x1+(CellWidth/2.0), y2+(CellHeight/2.0)));	// Average the averages + centre
				avgClr = (avgL+avgR+avgC) / 3.0;				
			}
			else if ((x < y)&&(x < 1.0 - y))	{
				// Average bottom left, top left, center
				vec4 avgL = IMG_NORM_PIXEL(inputImage, vec2(x1, y1));
				vec4 avgR = IMG_NORM_PIXEL(inputImage, vec2(x1, y2));
				vec4 avgC = IMG_NORM_PIXEL(inputImage, vec2(x1+(CellWidth/2.0), y2+(CellHeight/2.0)));	// Average the averages + centre
				avgClr = (avgL+avgR+avgC) / 3.0;
			}
			else if ((x > 1.0 - y)&&(x < y))	{
				// Average top left, top right, center
				vec4 avgL = IMG_NORM_PIXEL(inputImage, vec2(x1, y2));
				vec4 avgR = IMG_NORM_PIXEL(inputImage, vec2(x2, y2));
				vec4 avgC = IMG_NORM_PIXEL(inputImage, vec2(x1+(CellWidth/2.0), y2+(CellHeight/2.0)));	// Average the averages + centre
				avgClr = (avgL+avgR+avgC) / 3.0;
				//avgClr = vec4(0.0,1.0,0.0,1.0);
			}
			else	{
				// Average top right, bottom right, center
				vec4 avgL = IMG_NORM_PIXEL(inputImage, vec2(x2, y1));
				vec4 avgR = IMG_NORM_PIXEL(inputImage, vec2(x2, y2));
				vec4 avgC = IMG_NORM_PIXEL(inputImage, vec2(x1+(CellWidth/2.0), y2+(CellHeight/2.0)));	// Average the averages + centre
				avgClr = (avgL+avgR+avgC) / 3.0;
				//avgClr = vec4(0.0,0.0,1.0,1.0);
			}
		}
		
		gl_FragColor = avgClr;
	}
}
