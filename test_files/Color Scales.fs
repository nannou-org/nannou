/*
{
  "CATEGORIES" : [
    "Color"
  ],
  "DESCRIPTION" : "",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "LABELS" : [
        "Comparison Mode",
        "Newton",
        "Castel",
        "Field",
        "Jameson",
        "Seemann",
        "Rimington",
        "Bishop",
        "Helmholtz",
        "Scriabin",
        "Klein",
        "Aeppli",
        "Belmont",
        "Zieverink"
      ],
      "NAME" : "iAuthor",
      "TYPE" : "long",
      "DEFAULT" : 0,
      "LABEL" : "Author",
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
        13
      ]
    },
    {
      "NAME" : "iNote",
      "TYPE" : "float",
      "MAX" : 11,
      "DEFAULT" : 0,
      "MIN" : 0,
      "LABEL" : "Note"
    },
    {
      "NAME" : "iPreviewMode",
      "TYPE" : "bool",
      "DEFAULT" : 0
    }
  ],
  "CREDIT" : "VIDVOX"
}
*/


/*
//	Color Scales via http://rhythmiclight.com/archives/ideas/colorscales.html
C C# D D# E F F# G G# A A# B 
Newton • red • orange • yellow • green • blue • indigo • violet (Gerstner, p.167)
Castel • blue • blue-green • green • olive green • yellow • yellow-orange • orange • red • crimson • violet • agate • indigo (Peacock, p.400)
Field • blue • purple • red • orange • yellow • yellow green • green (Klein, p.69)
Jameson • red • red-orange • orange • orange-yellow • yellow • green • green-blue • blue • blue-purple • purple • purple-violet • violet (Jameson, p.12)
Seemann • carmine • scarlet • orange • yellow-orange • yellow • green • green blue • blue • indigo • violet • brown • black (Klein, p.86)
Rimington • deep red • crimson • orange-crimson • orange • yellow • yellow-green • green • blueish green • blue-green • indigo • deep blue • violet (Peacock, p.402)
Bishop • red • orange-red or scarlet • orange • gold or yellow-orange • yellow or green-gold • yellow-green • green • greenish-blue or aquamarine • blue • indigo or violet-blue • violet • violet-red • red (Bishop, p.11)
Helmholtz • yellow • green • greenish blue • cayan-blue • indigo blue • violet • end of red • red • red • red • red orange • orange (Helmholtz, p.22)
Scriabin • red • violet • yellow • steely with the glint of metal • pearly blue the shimmer of moonshine • dark red • bright blue • rosy orange • purple • green • steely with a glint of metal • pearly blue the shimmer of moonshine (Jones, p.104)
Klein • dark red • red • red orange • orange • yellow • yellow green • green • blue-green • blue • blue violet • violet • dark violet (Klein, p.209)
Aeppli • red • orange • yellow • green • blue-green • ultramarine blue • violet • purple (Gerstner, p.169)
Belmont • red • red-orange • orange • yellow-orange • yellow • yellow-green • green • blue-green • blue • blue-violet • violet • red-violet (Belmont, p.226)
Zieverink • yellow/green • green • blue/green • blue • indigo • violet • ultra violet • infra red • red • orange • yellow/white • yellow (Cincinnati Contemporary Art Center)
*/

const vec3 red = vec3(231.0/255.0,52.0/255.0,39.0/255.0);
const vec3 orange = vec3(234.0/255.0,134.0/255.0,54.0/255.0);
const vec3 yellow = vec3(245.0/255.0,243.0/255.0,98.0/255.0);
const vec3 green = vec3(64.0/255.0,141.0/255.0,64.0/255.0);
const vec3 blue = vec3(25.0/255.0,19.0/255.0,125.0/255.0);

const vec3 indigo = vec3(117.0/255.0,28.0/255.0,120.0/255.0);
const vec3 violet = vec3(199.0/255.0,49.0/255.0,132.0/255.0);

const vec3 bluegreen = vec3(66.0/255.0,142.0/255.0,129.0/255.0);
const vec3 olivegreen = vec3(119.0/255.0,144.0/255.0,57.0/255.0);
const vec3 yelloworange = vec3(240.0/255.0,210.0/255.0,90.0/255.0);
const vec3 crimson = vec3(148.0/255.0,33.0/255.0,23.0/255.0);
const vec3 agate = vec3(69.0/255.0,23.0/255.0,120.0/255.0);

const vec3 purple = vec3(117.0/255.0,28.0/255.0,120.0/255.0);
const vec3 yellowgreen = vec3(119.0/255.0,144.0/255.0,57.0/255.0);
const vec3 redorange = vec3(227.0/255.0,85.0/255.0,44.0/255.0);
const vec3 orangeyellow = vec3(240.0/255.0,210.0/255.0,90.0/255.0);
const vec3 greenblue = vec3(66.0/255.0,142.0/255.0,129.0/255.0);
const vec3 bluepurple = vec3(69.0/255.0,23.0/255.0,120.0/255.0);
const vec3 purpleviolet = vec3(153.0/255.0,41.0/255.0,130.0/255.0);
const vec3 carmine = vec3(98.0/255.0,34.0/255.0,31.0/255.0);
const vec3 scarlet = vec3(231.0/255.0,52.0/255.0,39.0/255.0);
const vec3 brown = vec3(98.0/255.0,34.0/255.0,31.0/255.0);
const vec3 black = vec3(7.0/255.0);
const vec3 deepred = vec3(231.0/255.0,52.0/255.0,39.0/255.0);
const vec3 orangecrimson = vec3(227.0/255.0,85.0/255.0,44.0/255.0);
const vec3 blueishgreen = vec3(79.0/255.0,161.0/255.0,131.0/255.0);
const vec3 deepblue = vec3(25.0/255.0,19.0/255.0,125.0/255.0);

const vec3 greenishblue = vec3(66.0/255.0,142.0/255.0,129.0/255.0);
const vec3 violetred = vec3(201.0/255.0,51.0/255.0,83.0/255.0);
const vec3 darkviolet = vec3(117.0/255.0,28.0/255.0,120.0/255.0);
const vec3 cayanblue = vec3(45.0/255.0,91.0/255.0,155.0/255.0);
const vec3 indigoblue = purple;

const vec3 endofred = vec3(145.0/255.0,34.0/255.0,84.0/255.0);
const vec3 steely = vec3(89.0/255.0,87.0/255.0,130.0/255.0);
const vec3 pearlyblue = vec3(45.0/255.0,91.0/255.0,155.0/255.0);
const vec3 darkred = vec3(148.0/255.0,33.0/255.0,23.0/255.0);

const vec3 brightblue = vec3(25.0/255.0,19.0/255.0,125.0/255.0);
const vec3 rosyorange = vec3(234.0,134.0,54.0);
const vec3 ultramarineblue = vec3(45.0/255.0,91.0/255.0,155.0/255.0);
const vec3 blueviolet = vec3(69.0/255.0,23.0/255.0,120.0/255.0);

const vec3 ultraviolet = vec3(102.0/255.0,25.0/255.0,68.0/255.0);
const vec3 infrared = vec3(148.0/255.0,33.0/255.0,23.0/255.0);
const vec3 yellowwhite = vec3(238.0/255.0,239.0/255.0,149.0/255.0);


vec3 colorForAuthorAndNote(int author, int note)	{
	vec3	inputPixelColor = vec3(0.0);
	//	newton
	if (author == 0)	{
		if (note == 0)
			inputPixelColor.rgb = red;
		else if (note == 1)
			inputPixelColor.rgb = red;
		else if (note == 2)
			inputPixelColor.rgb = orange;
		else if (note == 3)
			inputPixelColor.rgb = orange;
		else if (note == 4)
			inputPixelColor.rgb = yellow;
		else if (note == 5)
			inputPixelColor.rgb = green;
		else if (note == 6)
			inputPixelColor.rgb = green;
		else if (note == 7)
			inputPixelColor.rgb = blue;
		else if (note == 8)
			inputPixelColor.rgb = blue;
		else if (note == 9)
			inputPixelColor.rgb = indigo;
		else if (note == 10)
			inputPixelColor.rgb = indigo;
		else if (note == 11)
			inputPixelColor.rgb = violet;
	}
	//	castel
	else if (author == 1)	{
		if (note == 0)
			inputPixelColor.rgb = blue;
		else if (note == 1)
			inputPixelColor.rgb = bluegreen;
		else if (note == 2)
			inputPixelColor.rgb = green;
		else if (note == 3)
			inputPixelColor.rgb = olivegreen;
		else if (note == 4)
			inputPixelColor.rgb = yellow;
		else if (note == 5)
			inputPixelColor.rgb = yelloworange;
		else if (note == 6)
			inputPixelColor.rgb = orange;
		else if (note == 7)
			inputPixelColor.rgb = red;
		else if (note == 8)
			inputPixelColor.rgb = crimson;
		else if (note == 9)
			inputPixelColor.rgb = violet;
		else if (note == 10)
			inputPixelColor.rgb = agate;
		else if (note == 11)
			inputPixelColor.rgb = indigo;
	}
	//	field
	else if (author == 2)	{
		if (note == 0)
			inputPixelColor.rgb = blue;
		else if (note == 1)
			inputPixelColor.rgb = blue;
		else if (note == 2)
			inputPixelColor.rgb = purple;
		else if (note == 3)
			inputPixelColor.rgb = purple;
		else if (note == 4)
			inputPixelColor.rgb = red;
		else if (note == 5)
			inputPixelColor.rgb = orange;
		else if (note == 6)
			inputPixelColor.rgb = orange;
		else if (note == 7)
			inputPixelColor.rgb = yellow;
		else if (note == 8)
			inputPixelColor.rgb = yellow;
		else if (note == 9)
			inputPixelColor.rgb = yellowgreen;
		else if (note == 10)
			inputPixelColor.rgb = yellowgreen;
		else if (note == 11)
			inputPixelColor.rgb = green;
	}
	//	jameson
	else if (author == 3)	{
		if (note == 0)
			inputPixelColor.rgb = red;
		else if (note == 1)
			inputPixelColor.rgb = redorange;
		else if (note == 2)
			inputPixelColor.rgb = orange;
		else if (note == 3)
			inputPixelColor.rgb = orangeyellow;
		else if (note == 4)
			inputPixelColor.rgb = yellow;
		else if (note == 5)
			inputPixelColor.rgb = green;
		else if (note == 6)
			inputPixelColor.rgb = greenblue;
		else if (note == 7)
			inputPixelColor.rgb = blue;
		else if (note == 8)
			inputPixelColor.rgb = bluepurple;
		else if (note == 9)
			inputPixelColor.rgb = purple;
		else if (note == 10)
			inputPixelColor.rgb = purpleviolet;
		else if (note == 11)
			inputPixelColor.rgb = violet;
	}
	//	seemann
	else if (author == 4)	{
		if (note == 0)
			inputPixelColor.rgb = carmine;
		else if (note == 1)
			inputPixelColor.rgb = scarlet;
		else if (note == 2)
			inputPixelColor.rgb = orange;
		else if (note == 3)
			inputPixelColor.rgb = yelloworange;
		else if (note == 4)
			inputPixelColor.rgb = yellow;
		else if (note == 5)
			inputPixelColor.rgb = green;
		else if (note == 6)
			inputPixelColor.rgb = greenblue;
		else if (note == 7)
			inputPixelColor.rgb = blue;
		else if (note == 8)
			inputPixelColor.rgb = indigo;
		else if (note == 9)
			inputPixelColor.rgb = violet;
		else if (note == 10)
			inputPixelColor.rgb = brown;
		else if (note == 11)
			inputPixelColor.rgb = black;
	}
	//	rimington
	else if (author == 5)	{
		if (note == 0)
			inputPixelColor.rgb = deepred;
		else if (note == 1)
			inputPixelColor.rgb = crimson;
		else if (note == 2)
			inputPixelColor.rgb = orangecrimson;
		else if (note == 3)
			inputPixelColor.rgb = orange;
		else if (note == 4)
			inputPixelColor.rgb = yellow;
		else if (note == 5)
			inputPixelColor.rgb = yellowgreen;
		else if (note == 6)
			inputPixelColor.rgb = green;
		else if (note == 7)
			inputPixelColor.rgb = blueishgreen;
		else if (note == 8)
			inputPixelColor.rgb = bluegreen;
		else if (note == 9)
			inputPixelColor.rgb = indigo;
		else if (note == 10)
			inputPixelColor.rgb = deepblue;
		else if (note == 11)
			inputPixelColor.rgb = violet;
	}
	//	bishop
	else if (author == 6)	{
		if (note == 0)
			inputPixelColor.rgb = red;
		else if (note == 1)
			inputPixelColor.rgb = scarlet;
		else if (note == 2)
			inputPixelColor.rgb = orange;
		else if (note == 3)
			inputPixelColor.rgb = yelloworange;
		else if (note == 4)
			inputPixelColor.rgb = yellow;
		else if (note == 5)
			inputPixelColor.rgb = yellowgreen;
		else if (note == 6)
			inputPixelColor.rgb = green;
		else if (note == 7)
			inputPixelColor.rgb = greenishblue;
		else if (note == 8)
			inputPixelColor.rgb = blue;
		else if (note == 9)
			inputPixelColor.rgb = indigo;
		else if (note == 10)
			inputPixelColor.rgb = violet;
		else if (note == 11)
			inputPixelColor.rgb = violetred;
	}
	//	helmholtz
	else if (author == 7)	{
		if (note == 0)
			inputPixelColor.rgb = yellow;
		else if (note == 1)
			inputPixelColor.rgb = green;
		else if (note == 2)
			inputPixelColor.rgb = greenishblue;
		else if (note == 3)
			inputPixelColor.rgb = cayanblue;
		else if (note == 4)
			inputPixelColor.rgb = indigoblue;
		else if (note == 5)
			inputPixelColor.rgb = violet;
		else if (note == 6)
			inputPixelColor.rgb = endofred;
		else if (note == 7)
			inputPixelColor.rgb = red;
		else if (note == 8)
			inputPixelColor.rgb = red;
		else if (note == 9)
			inputPixelColor.rgb = red;
		else if (note == 10)
			inputPixelColor.rgb = redorange;
		else if (note == 11)
			inputPixelColor.rgb = orange;
	}
	//	scriabin
	else if (author == 8)	{
		if (note == 0)
			inputPixelColor.rgb = red;
		else if (note == 1)
			inputPixelColor.rgb = violet;
		else if (note == 2)
			inputPixelColor.rgb = yellow;
		else if (note == 3)
			inputPixelColor.rgb = steely;
		else if (note == 4)
			inputPixelColor.rgb = pearlyblue;
		else if (note == 5)
			inputPixelColor.rgb = darkred;
		else if (note == 6)
			inputPixelColor.rgb = brightblue;
		else if (note == 7)
			inputPixelColor.rgb = rosyorange;
		else if (note == 8)
			inputPixelColor.rgb = purple;
		else if (note == 9)
			inputPixelColor.rgb = green;
		else if (note == 10)
			inputPixelColor.rgb = steely;
		else if (note == 11)
			inputPixelColor.rgb = pearlyblue;
	}
	//	klein
	else if (author == 9)	{
		if (note == 0)
			inputPixelColor.rgb = darkred;
		else if (note == 1)
			inputPixelColor.rgb = red;
		else if (note == 2)
			inputPixelColor.rgb = redorange;
		else if (note == 3)
			inputPixelColor.rgb = orange;
		else if (note == 4)
			inputPixelColor.rgb = yellow;
		else if (note == 5)
			inputPixelColor.rgb = yellowgreen;
		else if (note == 6)
			inputPixelColor.rgb = green;
		else if (note == 7)
			inputPixelColor.rgb = bluegreen;
		else if (note == 8)
			inputPixelColor.rgb = blue;
		else if (note == 9)
			inputPixelColor.rgb = blueviolet;
		else if (note == 10)
			inputPixelColor.rgb = violet;
		else if (note == 11)
			inputPixelColor.rgb = darkviolet;
	}
	//	aeppli
	else if (author == 10)	{
		if (note == 0)
			inputPixelColor.rgb = red;
		else if (note == 1)
			inputPixelColor.rgb = red;
		else if (note == 2)
			inputPixelColor.rgb = orange;
		else if (note == 3)
			inputPixelColor.rgb = orange;
		else if (note == 4)
			inputPixelColor.rgb = yellow;
		else if (note == 5)
			inputPixelColor.rgb = yellow;
		else if (note == 6)
			inputPixelColor.rgb = green;
		else if (note == 7)
			inputPixelColor.rgb = bluegreen;
		else if (note == 8)
			inputPixelColor.rgb = bluegreen;
		else if (note == 9)
			inputPixelColor.rgb = ultramarineblue;
		else if (note == 10)
			inputPixelColor.rgb = violet;
		else if (note == 11)
			inputPixelColor.rgb = purple;
	}
	//	belmont
	else if (author == 11)	{
		if (note == 0)
			inputPixelColor.rgb = red;
		else if (note == 1)
			inputPixelColor.rgb = redorange;
		else if (note == 2)
			inputPixelColor.rgb = orange;
		else if (note == 3)
			inputPixelColor.rgb = yelloworange;
		else if (note == 4)
			inputPixelColor.rgb = yellow;
		else if (note == 5)
			inputPixelColor.rgb = yellowgreen;
		else if (note == 6)
			inputPixelColor.rgb = green;
		else if (note == 7)
			inputPixelColor.rgb = bluegreen;
		else if (note == 8)
			inputPixelColor.rgb = blue;
		else if (note == 9)
			inputPixelColor.rgb = blueviolet;
		else if (note == 10)
			inputPixelColor.rgb = violet;
		else if (note == 11)
			inputPixelColor.rgb = violetred;
	}
	//	zieverink
	else if (author == 12)	{
		if (note == 0)
			inputPixelColor.rgb = yellowgreen;
		else if (note == 1)
			inputPixelColor.rgb = green;
		else if (note == 2)
			inputPixelColor.rgb = bluegreen;
		else if (note == 3)
			inputPixelColor.rgb = blue;
		else if (note == 4)
			inputPixelColor.rgb = indigo;
		else if (note == 5)
			inputPixelColor.rgb = violet;
		else if (note == 6)
			inputPixelColor.rgb = ultraviolet;
		else if (note == 7)
			inputPixelColor.rgb = infrared;
		else if (note == 8)
			inputPixelColor.rgb = red;
		else if (note == 9)
			inputPixelColor.rgb = orange;
		else if (note == 10)
			inputPixelColor.rgb = yellowwhite;
		else if (note == 11)
			inputPixelColor.rgb = yellow;
	}
	return inputPixelColor;
}

void main()	{
	vec4		inputPixelColor = vec4(1.0);
	float		mixPoint = (iPreviewMode) ? 0.0 : fract(iNote);
	int			note1 = (iPreviewMode) ? int(floor((12.0*isf_FragNormCoord.x))) : int(iNote);
	int			note2 = (mixPoint == 0.0) ? note1 : note1 + 1;
	int			author = (iAuthor == 0) ? int(floor((13.0*(1.0-isf_FragNormCoord.y)))) : iAuthor - 1;
	
	vec3		c1 = colorForAuthorAndNote(author,note1);
	vec3		c2 = colorForAuthorAndNote(author,note2);
	
	inputPixelColor.rgb = mix(c1,c2,mixPoint);
	
	gl_FragColor = inputPixelColor;
}
