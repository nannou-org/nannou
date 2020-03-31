/*{
    "CATEGORIES": [
        "Geometry Adjustment"
    ],
    "CREDIT": "by VIDVOX",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0.01,
            "NAME": "seed",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.125,
            "MAX": 1,
            "MIN": 0.01,
            "NAME": "cell_size",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "NAME": "allow_flips_h",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0,
            "NAME": "allow_flips_v",
            "TYPE": "bool"
        }
    ],
    "ISFVSN": "2"
}
*/

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
		float CellWidth = cell_size;
		float CellHeight = cell_size;
		
		/*
		float x1 = floor(xy.x / CellWidth)*CellWidth;
		float x2 = clamp((ceil(xy.x / CellWidth)*CellWidth), 0.0, 1.0);
		// Top and bottom of tile
		float y1 = floor(xy.y / CellHeight)*CellHeight;
		float y2 = clamp((ceil(xy.y / CellHeight)*CellHeight), 0.0, 1.0);
		*/
		
		//	divide 1 by the cell width and cell height to determine the count
		float rows = floor(1.0/CellHeight);
		float cols = floor(1.0/CellWidth);
		float count = floor(rows * cols);
		
		//	figure out the ID # of the region
		float region = cols*floor(xy.x / CellWidth) + floor(xy.y / CellHeight);

		//	use this to draw the gradient of the regions as gray colors..
		//gl_FragColor = vec4(vec3(region/count),1.0);
		
		//	now translate this region to another random region using our seed and region
		float translated = clamp(rand(vec2(region/count, seed)),0.0,1.0);
		//translated = region/count;
		//gl_FragColor = vec4(vec3(translated),1.0);
		
		//	quantize the translated!
		translated = floor(count * translated);
		//gl_FragColor = vec4(vec3(translated),1.0);
		//	now convert the translated region back to an xy location
		//	get the relative position within the original block and then add on the translated amount
		xy.x = (xy.x - floor(xy.x / CellWidth)*CellWidth) + CellWidth * floor(translated / rows);
		//xy.x = (xy.x - floor(xy.x / CellWidth)*CellWidth);
		xy.y = xy.y - floor(xy.y / CellHeight)*CellHeight + CellHeight * floor(mod(translated , cols));
		
		//	lastly if flips are allowed, randomly flip h
		if (allow_flips_h)	{
			float flipx = rand(vec2(translated, seed));
			if (flipx > 0.5)	{
				xy.x = 1.0-xy.x;
			}
		}
		if (allow_flips_v)	{
			float flipy = rand(vec2(translated, seed));
			if (flipy > 0.5)	{
				xy.y = 1.0-xy.y;
			}
		}
		
		gl_FragColor = IMG_NORM_PIXEL(inputImage, xy);
		
	}
}