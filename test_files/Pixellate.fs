
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
			"NAME": "shape",
			"VALUES": [
				0,
				1
			],
			"LABELS": [
				"Square",
				"Rectangle"
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
		if (shape==0)	{
			CellHeight = cell_size * RENDERSIZE.x / RENDERSIZE.y;
		}

		float x1 = floor(xy.x / CellWidth)*CellWidth;
		float x2 = clamp((ceil(xy.x / CellWidth)*CellWidth), 0.0, 1.0);
		// Top and bottom of tile
		float y1 = floor(xy.y / CellHeight)*CellHeight;
		float y2 = clamp((ceil(xy.y / CellHeight)*CellHeight), 0.0, 1.0);

		// GET AVERAGE CELL COLOUR
		// Average left and right pixels
		vec4 avgX = (IMG_NORM_PIXEL(inputImage, vec2(x1, y1))+(IMG_NORM_PIXEL(inputImage, vec2(x2, y1)))) / 2.0;
		// Average top and bottom pixels
		vec4 avgY = (IMG_NORM_PIXEL(inputImage, vec2(x1, y1))+(IMG_NORM_PIXEL(inputImage, vec2(x1, y2)))) / 2.0;
		// Centre pixel
		vec4 avgC = IMG_NORM_PIXEL(inputImage, vec2(x1+(CellWidth/2.0), y2+(CellHeight/2.0)));	// Average the averages + centre
		vec4 avgClr = (avgX+avgY+avgC) / 3.0;

		gl_FragColor = vec4(avgClr);
	}
}
