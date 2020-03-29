/*{
    "CATEGORIES": [
        "Stylize"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.125,
            "MAX": 0.5,
            "MIN": 0.001,
            "NAME": "cell_size",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.05000000074505806,
            "MAX": 0.5,
            "MIN": 0.001,
            "NAME": "min_cell_size",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": 0,
            "NAME": "rSeed",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "LABELS": [
                "Square",
                "Rectangle"
            ],
            "NAME": "shape",
            "TYPE": "long",
            "VALUES": [
                0,
                1
            ]
        },
        {
            "DEFAULT": 2,
            "LABELS": [
                "Off",
                "2",
                "3",
                "5"
            ],
            "NAME": "round_to_divisions",
            "TYPE": "long",
            "VALUES": [
                0,
                2,
                3,
                5
            ]
        }
    ],
    "ISFVSN": "2"
}
*/

#ifndef GL_ES
float distance (vec2 center, vec2 pt)
{
	float tmp = pow(center.x-pt.x,2.0)+pow(center.y-pt.y,2.0);
	return pow(tmp,0.5);
}
#endif

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

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
		float max_cell_size = cell_size + min_cell_size;
		float CellWidth = max_cell_size;
		float CellHeight = max_cell_size;
		if (shape==0)	{
			CellHeight = max_cell_size * RENDERSIZE.x / RENDERSIZE.y;
		}

		float x1 = floor(xy.x / CellWidth)*CellWidth;
		// Top and bottom of tile
		float y1 = floor(xy.y / CellHeight)*CellHeight;
		
		float newCellSize = max_cell_size;
		
		if (round_to_divisions > 0)	{
			float maxCount = (min_cell_size > 0.0) ? max_cell_size / min_cell_size : 10.0;
			float val = floor(1.0+(maxCount-1.0)*rand(0.1+rSeed+vec2(x1,y1)));
			newCellSize = newCellSize / pow(float(round_to_divisions),val);
		}
		else	{
			newCellSize = min_cell_size + (cell_size - min_cell_size) * rand(0.1+rSeed+vec2(x1,y1));
		}
		
		CellWidth = newCellSize;
		CellHeight = newCellSize;
		if (shape==0)	{
			CellHeight = newCellSize * RENDERSIZE.x / RENDERSIZE.y;
		}
		
		x1 = floor(xy.x / CellWidth)*CellWidth;
		float x2 = clamp((ceil(xy.x / CellWidth)*CellWidth), 0.0, 1.0);
		// Top and bottom of tile
		y1 = floor(xy.y / CellHeight)*CellHeight;
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
