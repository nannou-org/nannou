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
      	"Custom Input",
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
		"NAME": "iBaseCustomColor",
		"TYPE": "color",
		"DEFAULT": [
			0.25,
			0.59,
			0.9,
			1.0
		]
	},
	{
		"NAME": "iColorMode",
		"TYPE": "long",
		"VALUES": [
			0,
			1,
			2,
			3,
			4,
			5,
			6
		],
		"LABELS": [
			"Basic Complementary",
			"Split Complementary",
			"Compound Complementary",
			"Spectrum",
			"Shades",
			"Analogous",
			"Compound Analogous"
		],
		"DEFAULT": 0
	},
    {
      "NAME" : "note0",
      "TYPE" : "bool",
      "DEFAULT" : 0
    },
    {
      "NAME" : "note1",
      "TYPE" : "bool",
      "DEFAULT" : 0
    },
    {
      "NAME" : "note2",
      "TYPE" : "bool",
      "DEFAULT" : 0
    },
    {
      "NAME" : "note3",
      "TYPE" : "bool",
      "DEFAULT" : 0
    },
    {
      "NAME" : "note4",
      "TYPE" : "bool",
      "DEFAULT" : 0
    },
    {
      "NAME" : "note5",
      "TYPE" : "bool",
      "DEFAULT" : 0
    },
    {
      "NAME" : "note6",
      "TYPE" : "bool",
      "DEFAULT" : 0
    },
    {
      "NAME" : "note7",
      "TYPE" : "bool",
      "DEFAULT" : 0
    },
    {
      "NAME" : "note8",
      "TYPE" : "bool",
      "DEFAULT" : 0
    },
    {
      "NAME" : "note9",
      "TYPE" : "bool",
      "DEFAULT" : 0
    },
    {
      "NAME" : "note10",
      "TYPE" : "bool",
      "DEFAULT" : 0
    },
    {
      "NAME" : "note11",
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


vec4 colorForAuthorAndNote(int author, int note)	{
	vec4	inputPixelColor = vec4(0.0);
	inputPixelColor.a = 1.0;
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

vec3 rgb2hsv(vec3 c)	{
	vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	//vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
	//vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
	vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
	vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);
	
	float d = q.x - min(q.w, q.y);
	float e = 1.0e-10;
	return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)	{
	vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
	vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
	return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec4 relatedColorForMode(vec4 inColor, int colorMode, float index, int colorCount)	{
	vec3		outColor = rgb2hsv(inColor.rgb);
	float		variation = 0.3236;	//	1/5 the golden ratio
	
	//	Basic complimentary – saturation and brightness variations on two fixed 180 degree opposite hues
	if (colorMode == 0)	{
		if (mod(index, 2.0) >= 1.0)	{
			outColor.r = outColor.r + 0.5;
			outColor.r = outColor.r - floor(outColor.r);
		}
		
		outColor.g = outColor.g - variation * floor(index / 2.0);
		
		if (outColor.g < 0.1)	{
			outColor.g = outColor.g + variation * floor(index / 2.0);
			outColor.g = outColor.g - floor(outColor.g);
		}
		
		outColor.b = outColor.b - variation * floor(index / 4.0);
		if (outColor.b < 0.2)	{
			outColor.b = outColor.b + variation * floor(index / 4.0);
			outColor.b = outColor.b - floor(outColor.b);
		}
	}
	//	Split complimentary – saturation and brightness variations on a 3 fixed 120 degree hues
	else if (colorMode == 1)	{
		float divisor = 3.0;
		float ratio = 0.45;
		if (mod(index, 3.0) >= 2.0)	{
			outColor.r = outColor.r - ratio;
		}
		else if (mod(index, 3.0) >= 1.0)	{
			outColor.r = outColor.r + ratio;
		}
		
		//outColor.g = outColor.g + variation * floor(index / divisor);
		
		if (mod(index, 5.0) >= 3.0)	{
			outColor.g = outColor.g - variation;
			outColor.g = outColor.g - floor(outColor.g);
		}
		outColor.b = outColor.b - variation * floor(index / (divisor));
		if (outColor.b < 0.1)	{
			outColor.b = outColor.b + variation * floor(index / (divisor));
			outColor.b = outColor.b - floor(outColor.b);
		}
	}
	//	Compound complimentary – a combination of shades, complimentary and analogous colors with slight shifts
	else if (colorMode == 2)	{
		if (mod(index, 3.0) >= 2.0)	{
			outColor.r = outColor.r + 0.5;
			outColor.r = outColor.r + variation * index * 1.0 / float(colorCount - 1) / 4.0;
		}
		else	{
			outColor.r = outColor.r + variation * index * 1.0 / float(colorCount - 1);
		}
		outColor.r = outColor.r - floor(outColor.r);
		
		
		if (mod(index, 2.0) >= 1.0)	{
			outColor.g = outColor.g + index * variation / 2.0;
		}
		else if (mod(index, 3.0) >= 2.0)	{
			outColor.g = outColor.g - variation / 2.0;
		}
		else	{
			outColor.g = outColor.g - index * variation / float(colorCount - 1);
		}
		if (outColor.g > 1.0)	{
			outColor.g = outColor.g - floor(outColor.g);
		}
	}
	//	Spectrum – hue shifts based on number of colors with minor saturation shifts
	else if (colorMode == 3)	{
		outColor.r = outColor.r + index * 1.0 / float(colorCount);
		if (mod(index, 3.0) >= 2.0)	{
			outColor.g = outColor.g - variation / 2.0;
			outColor.g = outColor.g - floor(outColor.g);
		}
		else if (mod(index, 4.0) >= 3.0)	{
			outColor.g = outColor.g + variation / 2.0;
			//outColor.g = outColor.g - floor(outColor.g);
		}
	}
	//	Shades – saturation and brightness variations on a single fixed hue
	else if (colorMode == 4)	{
		if (mod(index, 2.0) >= 1.0)	{
			outColor.b = outColor.b - (index * variation) / float(colorCount-1);
		}
		else	{
			outColor.b = outColor.b + (index * variation) / float(colorCount-1);
			outColor.b = outColor.b - floor(outColor.b);
		}
		if (outColor.b < 0.075)	{
			outColor.b = 1.0 - outColor.b * variation;
		}
		
		if (mod(index, 3.0) >= 2.0)	{
			outColor.g = outColor.g - (index * variation) / 2.0;
		}
		else if (mod(index, 4.0) >= 3.0)	{
			outColor.g = outColor.g + (index * variation) / 2.0;
		}
		
		if ((outColor.g > 1.0) || (outColor.g < 0.05))	{
			outColor.g = outColor.g - floor(outColor.g);
		}
	}
	//	Analogous – small hue and saturation shifts 
	else if (colorMode == 5)	{

		outColor.r = outColor.r + variation * index * 1.0 / float(colorCount - 1); 		

		if (mod(index, 3.0) >= 1.0)	{
			outColor.g = outColor.g - variation / 2.0;
			if (outColor.g < 0.0)	{
				outColor.g = outColor.g + variation / 2.0;
			}
			if (outColor.g > 1.0)	{
				outColor.g = outColor.g - floor(outColor.g);
			}
		}
	}
	//	Compound Analogous – similar to analogous but with negative hue shifts
	else if (colorMode == 6)	{
		if (mod(index, 3.0) >= 1.0)	{
			outColor.r = outColor.r + variation * index * 1.0 / float(colorCount - 1); 
		}
		else	{
			outColor.r = outColor.r - variation * index * 0.5 / float(colorCount - 1);	
		}
		if (mod(index, 3.0) >= 1.0)	{
			outColor.g = outColor.g - variation / 2.0;
			if (outColor.g < 0.0)	{
				outColor.g = outColor.g + variation / 2.0;
			}
			if (outColor.g > 1.0)	{
				outColor.g = outColor.g - floor(outColor.g);
			}
		}	
	}
	
	return vec4(hsv2rgb(outColor.rgb),inColor.a);
}

void main()	{
	vec4		inputPixelColor = vec4(0.0);
	vec4		colorsArray[12];
	int			colorsCount = 0;
	
	//	how many colors are active?
	if (note0)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,0.0,12) : colorForAuthorAndNote(iAuthor-1,0);
		++colorsCount;
	}
	if (note1)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,1.0,12) : colorForAuthorAndNote(iAuthor-1,1);
		++colorsCount;
	}
	if (note2)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,2.0,12) :  colorForAuthorAndNote(iAuthor-1,2);
		++colorsCount;
	}
	if (note3)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,3.0,12) :  colorForAuthorAndNote(iAuthor-1,3);
		++colorsCount;
	}
	if (note4)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,4.0,12) :  colorForAuthorAndNote(iAuthor-1,4);
		++colorsCount;
	}
	if (note5)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,5.0,12) :  colorForAuthorAndNote(iAuthor-1,5);
		++colorsCount;
	}
	if (note6)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,6.0,12) :  colorForAuthorAndNote(iAuthor-1,6);
		++colorsCount;
	}
	if (note7)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,7.0,12) :  colorForAuthorAndNote(iAuthor-1,7);
		++colorsCount;
	}
	if (note8)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,8.0,12) :  colorForAuthorAndNote(iAuthor-1,8);
		++colorsCount;
	}
	if (note9)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,9.0,12) :  colorForAuthorAndNote(iAuthor-1,9);
		++colorsCount;
	}
	if (note10)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,10.0,12) :  colorForAuthorAndNote(iAuthor-1,10);
		++colorsCount;
	}
	if (note11)	{
		colorsArray[colorsCount] = (iAuthor == 0) ? relatedColorForMode(iBaseCustomColor,iColorMode,11.0,12) :  colorForAuthorAndNote(iAuthor-1,11);
		++colorsCount;
	}
	
	if (colorsCount > 0)	{
		int		thisIndex = int(isf_FragNormCoord.x * float(colorsCount));
		inputPixelColor = colorsArray[thisIndex];
	}
	
	gl_FragColor = inputPixelColor;
}
