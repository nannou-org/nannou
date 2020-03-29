/*
{
  "CATEGORIES" : [
    "Pattern", "Color"
  ],
  "DESCRIPTION" : "Draws basic graph paper pattern",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "bgColor",
      "TYPE" : "color",
      "DEFAULT" : [
        0.93999999761581421,
        0.93999999761581421,
        0.97000002861022949,
        1
      ]
    },
    {
      "NAME" : "lineColor",
      "TYPE" : "color",
      "DEFAULT" : [
        0.63999998569488525,
        0.76999998092651367,
        0.95999997854232788,
        1
      ]
    },
    {
      "LABELS" : [
        "0",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        "10",
        "11",
        "12",
        "13",
        "14",
        "15",
        "16"
      ],
      "NAME" : "majorDivisions",
      "TYPE" : "long",
      "DEFAULT" : 3,
      "VALUES" : [
        0,
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        9,
        10,
        11,
        12,
        13,
        14,
        15,
        16
      ]
    },
    {
      "LABELS" : [
        "0",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8"
      ],
      "NAME" : "minorHDivisions",
      "TYPE" : "long",
      "DEFAULT" : 2,
      "VALUES" : [
        0,
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8
      ]
    },
    {
      "LABELS" : [
        "0",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8"
      ],
      "NAME" : "minorVDivisions",
      "TYPE" : "long",
      "DEFAULT" : 2,
      "VALUES" : [
        0,
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8
      ]
    },
    {
      "NAME" : "majorDivisionLineWidth",
      "TYPE" : "float",
      "MAX" : 5,
      "DEFAULT" : 3,
      "MIN" : 1
    },
    {
      "NAME" : "square",
      "TYPE" : "bool",
      "DEFAULT" : true
    }
  ],
  "CREDIT" : "VIDVOX"
}
*/


const float minorDivisionLineWidth = 1.0;


void main()	{
	vec4		inputPixelColor = bgColor;
	vec2		loc = gl_FragCoord.xy;
	vec2		divisionSize = (square) ? vec2(max(RENDERSIZE.x,RENDERSIZE.y)) : RENDERSIZE;
	divisionSize = (divisionSize - majorDivisionLineWidth) / (1.0 + float(majorDivisions));
	vec2		modLoc = mod(loc,divisionSize);
	if ((modLoc.x < majorDivisionLineWidth)||(modLoc.y < majorDivisionLineWidth))	{
		inputPixelColor = lineColor;
	}
	if (minorHDivisions > 0)	{
		vec2	locDivisionSize = (divisionSize) / (1.0+float(minorHDivisions));
		modLoc = mod(loc,locDivisionSize);
		if (modLoc.x < minorDivisionLineWidth)	{
			inputPixelColor = lineColor;
		}
	}
	if (minorVDivisions > 0)	{
		vec2	locDivisionSize = (divisionSize) / (1.0+float(minorVDivisions));
		modLoc = mod(loc,locDivisionSize);
		if (modLoc.y < minorDivisionLineWidth)	{
			inputPixelColor = lineColor;
		}
	}

	gl_FragColor = inputPixelColor;
}
