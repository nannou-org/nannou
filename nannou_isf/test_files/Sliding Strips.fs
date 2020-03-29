/*
{
  "CATEGORIES" : [
    "Geometry Adjustment"
  ],
  "DESCRIPTION" : "",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "xShiftAmount",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0,
      "MIN" : 0
    },
    {
      "NAME" : "yShiftAmount",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0,
      "MIN" : 0
    },
    {
      "NAME" : "xTileSize",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.25,
      "MIN" : 0
    },
    {
      "NAME" : "yTileSize",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.25,
      "MIN" : 0
    }
  ],
  "CREDIT" : ""
}
*/

void main()	{
	vec2		loc = isf_FragNormCoord.xy;
	vec2		coords = vec2(0.0);
	coords.x = (xTileSize == 0.0) ? gl_FragCoord.x : floor(loc.x / yTileSize);
	coords.y = (yTileSize == 0.0) ? gl_FragCoord.y : floor(loc.y / xTileSize);
	
	vec2		maxCoords = vec2(0.0);
	maxCoords.x = (xTileSize == 0.0) ? RENDERSIZE.x : (1.0 / yTileSize);
	maxCoords.y = (yTileSize == 0.0) ? RENDERSIZE.y : (1.0 / xTileSize);
	vec2		shiftAmount = vec2(xShiftAmount, yShiftAmount);
	
	shiftAmount.x = shiftAmount.x + shiftAmount.x * coords.y / maxCoords.y;
	
	if (shiftAmount.x < 0.0)
		shiftAmount.x = 0.0;
	else if (shiftAmount.x > 1.0)
		shiftAmount.x = 1.0;
	
	loc.x = mod(loc.x + shiftAmount.x, 1.0);
	
	shiftAmount.y = shiftAmount.y + shiftAmount.y * coords.x / maxCoords.x;
	
	if (shiftAmount.y < 0.0)
		shiftAmount.y = 0.0;
	else if (shiftAmount.y > 1.0)
		shiftAmount.y = 1.0;
	
	loc.y = mod(loc.y + shiftAmount.y, 1.0);
	
	gl_FragColor = IMG_NORM_PIXEL(inputImage, loc);
}
