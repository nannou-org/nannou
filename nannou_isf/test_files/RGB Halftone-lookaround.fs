/*{
    "CATEGORIES": [
        "Halftone Effect",
        "Retro"
    ],
    "CREDIT": "by zoidberg",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 45,
            "MAX": 256,
            "MIN": 1,
            "NAME": "gridSize",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.15,
            "MAX": 1,
            "MIN": 0,
            "NAME": "smoothing",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2"
}
*/

vec4		gridRot = vec4(15.0, 45.0, 75.0, 0.0);
//vec4		gridRot = vec4(15.0, 75.0, 0.0, 45.0);
//vec4		gridRot = vec4(0.0, 0.0, 0.0, 0.0);



//	during calculation we find the closest dot to a frag, determine its size, and then determine the size of the four dots above/below/right/left of it. this array of offsets move "one left", "one up", "one right", and "one down"...
vec2		originOffsets[4];



void main() {
	//	a halftone is an overlapping series of grids of dots
	//	each grid of dots is rotated by a different amount
	//	the size of the dots determines the colors. the shape of the dot should never change (always be a dot with regular edges)
	originOffsets[0] = vec2(-1.0, 0.0);
	originOffsets[1] = vec2(0.0, 1.0);
	originOffsets[2] = vec2(1.0, 0.0);
	originOffsets[3] = vec2(0.0, -1.0);
	
	vec3		rgbAmounts = vec3(0.0);
	int i;
	int j;
	//	for each of the channels (i) of RGB...
	for (i=0; i<3; ++i)	{
		//	figure out the rotation of the grid in radians
		float		rotRad = radians(gridRot[i]);
		//	the grids are rotated counter-clockwise- to find the nearest dot, take the fragment pixel loc, 
		//	rotate it clockwise, and split by the grid to find the center of the dot. then rotate this 
		//	coord counter-clockwise to yield the location of the center of the dot in pixel coords local to the render space
		mat2		ccTrans = mat2(vec2(cos(rotRad), sin(rotRad)), vec2(-1.0*sin(rotRad), cos(rotRad)));
		mat2		cTrans = mat2(vec2(cos(rotRad), -1.0*sin(rotRad)), vec2(sin(rotRad), cos(rotRad)));
		
		//	find the location of the frag in the grid (prior to rotating it)
		vec2		gridFragLoc = cTrans * gl_FragCoord.xy;
		//	find the center of the dot closest to the frag- there's no "round" in GLSL 1.2, so do a "floor" to find the dot to the bottom-left of the frag, then figure out if the frag would be in the top and right halves of that square to find the closest dot to the frag
		vec2		gridOriginLoc = vec2(floor(gridFragLoc.x/gridSize), floor(gridFragLoc.y/gridSize));
		
		vec2		tmpGridCoords = gridFragLoc/vec2(gridSize);
		bool		fragAtTopOfGrid = ((tmpGridCoords.y-floor(tmpGridCoords.y)) > (gridSize/2.0)) ? true : false;
		bool		fragAtRightOfGrid = ((tmpGridCoords.x-floor(tmpGridCoords.x)) > (gridSize/2.0)) ? true : false;
		if (fragAtTopOfGrid)
			gridOriginLoc.y = gridOriginLoc.y + 1.0;
		if (fragAtRightOfGrid)
			gridOriginLoc.x = gridOriginLoc.x + 1.0;
		//	...at this point, "gridOriginLoc" contains the grid coords of the nearest dot to the fragment being rendered
		//	convert the location of the center of the dot from grid coords to pixel coords
		vec2		gridDotLoc = vec2(gridOriginLoc.x*gridSize, gridOriginLoc.y*gridSize) + vec2(gridSize/2.0);
		//	rotate the pixel coords of the center of the dot so they become relative to the rendering space
		vec2		renderDotLoc = ccTrans * gridDotLoc;
		//	get the color of the pixel of the input image under this dot (the color will ultimately determine the size of the dot)
		vec4		renderDotImageColorRGB = IMG_PIXEL(inputImage, renderDotLoc);
		
		
		//	the amount of this channel is taken from the same channel of the color of the pixel of the input image under this halftone dot
		float		imageChannelAmount = renderDotImageColorRGB[i];
		//	the size of the dot is determined by the value of the channel
		float		dotRadius = imageChannelAmount * (gridSize*1.50/2.0);
		float		fragDistanceToDotCenter = distance(gl_FragCoord.xy, renderDotLoc);
		if (fragDistanceToDotCenter < dotRadius)	{
			rgbAmounts[i] += smoothstep(dotRadius, dotRadius-(dotRadius*smoothing), fragDistanceToDotCenter);
		}
		
		//	calcluate the size of the dots abov/below/to the left/right to see if they're overlapping
		for (j=0; j<4; ++j)	{
			gridDotLoc = vec2((gridOriginLoc.x+originOffsets[j].x)*gridSize, (gridOriginLoc.y+originOffsets[j].y)*gridSize) + vec2(gridSize/2.0);
			renderDotLoc = ccTrans * gridDotLoc;
			renderDotImageColorRGB = IMG_PIXEL(inputImage, renderDotLoc);
			
			imageChannelAmount = renderDotImageColorRGB[i];
			dotRadius = imageChannelAmount * (gridSize*1.50/2.0);
			fragDistanceToDotCenter = distance(gl_FragCoord.xy, renderDotLoc);
			if (fragDistanceToDotCenter < dotRadius)	{
				rgbAmounts[i] += smoothstep(dotRadius, dotRadius-(dotRadius*smoothing), fragDistanceToDotCenter);
			}
		}
	}
	
	gl_FragColor = vec4(rgbAmounts[0], rgbAmounts[1], rgbAmounts[2], 1.0);
}