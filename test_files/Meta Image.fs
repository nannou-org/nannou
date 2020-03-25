/*{
	"CREDIT": "by Toneburst",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Stylize", "Tile Effect"
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
			"MAX": 1.0,
			"DEFAULT": 0.125
		},
		{
			"NAME": "zoom_tile",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 2.0,
			"DEFAULT": 1.0
		},
		{
			"NAME": "mixAmt",
			"LABEL": "mix",
			"TYPE": "float",
			"MIN": 0.0,
			"MAX": 1.0,
			"DEFAULT": 0.5
		},
		{
			"NAME": "mode",
			"VALUES": [
				0,
				1
			],
			"LABELS": [
				"Multiply",
				"Mix"
			],
			"DEFAULT": 0,
			"TYPE": "long"
		}
	]
}*/

void main()
{
// CALCULATE EDGES OF CURRENT CELL
	// Position of current pixel
	vec2 xy = gl_FragCoord.xy / RENDERSIZE.xy;
	// Left and right of tile
	float CellWidth = cell_size;
	float CellHeight = cell_size;
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

	// GET PIXELS FROM LITTLE IMAGE	
	// X-position in current cell
	float cellPosX = (xy.x - x1) / CellWidth;
	// Y-position in current cell
	float cellPosY = (xy.y - y1) / CellHeight;
	
	vec2 loc = vec2(cellPosX, cellPosY);
	vec2 modifiedCenter = vec2(0.5);
	loc.x = (loc.x - modifiedCenter.x)*(1.0/zoom_tile) + modifiedCenter.x;
	loc.y = (loc.y - modifiedCenter.y)*(1.0/zoom_tile) + modifiedCenter.y;
	
	vec4 littlePix;
	if ((loc.x < 0.0)||(loc.y < 0.0)||(loc.x > 1.0)||(loc.y > 1.0))	{
		littlePix = vec4(0.0);
	}
	else	{
		littlePix = IMG_NORM_PIXEL(inputImage, loc);
	}
	
	// MULTIPLY LITTLE IMAGE COLOUR WITH AVERAGE CELL COLOUR AND OUTPUT
	if (mode == 0)	{
		gl_FragColor = vec4(littlePix * avgClr);
	}
	else	{
		gl_FragColor = vec4(mix(littlePix, avgClr, mixAmt));
	}
}
