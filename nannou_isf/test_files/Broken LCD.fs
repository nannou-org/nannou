/*
{
  "CATEGORIES" : [
    "Glitch",
    "Stylize"
  ],
  "DESCRIPTION" : "Broken LCD using value noise",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "flickerLevel",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0,
      "MIN" : 0
    },
    {
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
        9
      ],
      "LABELS" : [
        "Random1",
        "Random2",
        "Random3",
        "Nothing",
        "Noise",
        "Lines",
        "Vertical",
        "Horizontal",
        "Stripes",
        "Checkerboard"
      ],
      "IDENTITY" : 0,
      "DEFAULT" : 0,
      "LABEL" : "Pattern Style",
      "TYPE" : "long",
      "NAME" : "patternStyle"
    },
    {
      "NAME" : "glitchBrightness",
      "TYPE" : "float",
      "MAX" : 4,
      "DEFAULT" : 3,
      "MIN" : 0,
      "LABEL" : "Glitch Thickness"
    },
    {
      "NAME" : "glitchBrightnessCurve",
      "TYPE" : "float",
      "MAX" : 4,
      "DEFAULT" : 2,
      "LABEL" : "Glitch Shape",
      "MIN" : 1
    },
    {
      "NAME" : "glitchRadius",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.5,
      "LABEL" : "Glitch Spread",
      "MIN" : 0
    },
    {
      "NAME" : "glitchSmoothness",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.5,
      "MIN" : 0,
      "LABEL" : "Glitch Edge Smoothness"
    },
    {
      "NAME" : "glitchScale",
      "TYPE" : "float",
      "MAX" : 100,
      "DEFAULT" : 4.1013007164001465,
      "MIN" : 0,
      "LABEL" : "Glitch Size"
    },
    {
      "NAME" : "noiseSeed",
      "TYPE" : "float",
      "MAX" : 10,
      "DEFAULT" : 5.0863485336303711,
      "MIN" : 0,
      "LABEL" : "Glitch Seed"
    },
    {
      "NAME" : "patternSeed",
      "TYPE" : "float",
      "MAX" : 10,
      "DEFAULT" : 2.3372085094451904,
      "LABEL" : "Pattern Seed",
      "MIN" : 0
    },
    {
      "NAME" : "tintBrightness",
      "TYPE" : "float",
      "MAX" : 4,
      "DEFAULT" : 4,
      "MIN" : 0
    },
    {
      "NAME" : "tintRadius",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.22620739042758942,
      "MIN" : 0
    },
    {
      "NAME" : "tintBrightnessCurve",
      "TYPE" : "float",
      "MAX" : 4,
      "DEFAULT" : 1.2606533765792847,
      "MIN" : 1
    },
    {
      "NAME" : "tintSeed",
      "TYPE" : "float",
      "MAX" : 20,
      "DEFAULT" : 0,
      "MIN" : 0
    },
    {
      "NAME" : "tintColor1",
      "TYPE" : "color",
      "DEFAULT" : [
        0.90196079015731812,
        0.25098040699958801,
        0.25098040699958801,
        1
      ]
    },
    {
      "NAME" : "tintColor2",
      "TYPE" : "color",
      "DEFAULT" : [
        0.25098040699958801,
        0.50196081399917603,
        0.90196079015731812,
        1
      ]
    },
    {
      "NAME" : "tintScale",
      "TYPE" : "float",
      "MAX" : 100,
      "DEFAULT" : 4,
      "MIN" : 0
    },
    {
      "NAME" : "rowGlitchLevel",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.0099999997764825821,
      "MIN" : 0
    },
    {
      "NAME" : "rowFlickerLevel",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0,
      "MIN" : 0
    },
    {
      "NAME" : "rowGlitchSeed",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.34099999070167542,
      "MIN" : 0
    },
    {
      "NAME" : "hardEdge",
      "TYPE" : "bool",
      "DEFAULT" : true,
      "LABEL" : "Hard Edge"
    },
    {
      "VALUES" : [
        0,
        1,
        2
      ],
      "LABELS" : [
        "Original",
        "Multiply",
        "Replace"
      ],
      "IDENTITY" : 0,
      "DEFAULT" : 0,
      "LABEL" : "Alpha Mode",
      "TYPE" : "long",
      "NAME" : "alphaMode"
    }
  ],
  "ISFVSN" : "2",
  "CREDIT" : "VIDVOX"
}
*/


const float pi = 3.1415926535897932384626433832795;


vec2 rotatePoint(vec2 pt, float angle, vec2 center)
{
	vec2 returnMe;
	float s = sin(angle * pi);
	float c = cos(angle * pi);

	returnMe = pt;

	// translate point back to origin:
	returnMe.x -= center.x;
	returnMe.y -= center.y;

	// rotate point
	float xnew = returnMe.x * c - returnMe.y * s;
	float ynew = returnMe.x * s + returnMe.y * c;

	// translate point back:
	returnMe.x = xnew + center.x;
	returnMe.y = ynew + center.y;
	return returnMe;
}

float sign(vec2 p1, vec2 p2, vec2 p3)
{
	return (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y);
}

bool PointInTriangle(vec2 pt, vec2 v1, vec2 v2, vec2 v3)
{
	bool b1, b2, b3;

	b1 = sign(pt, v1, v2) < 0.0;
	b2 = sign(pt, v2, v3) < 0.0;
	b3 = sign(pt, v3, v1) < 0.0;

	return ((b1 == b2) && (b2 == b3));
}

bool RotatedPointInTriangle(vec2 pt, vec2 v1, vec2 v2, vec2 v3, vec2 center)
{
	bool b1, b2, b3;
	
	vec2 v1r = v1;
	vec2 v2r = v2;
	vec2 v3r = v3;

	b1 = sign(pt, v1r, v2r) < 0.0;
	b2 = sign(pt, v2r, v3r) < 0.0;
	b3 = sign(pt, v3r, v1r) < 0.0;

	return ((b1 == b2) && (b2 == b3));
}


float isPointInShape(vec2 pt, int shape, vec4 shapeCoordinates)	{
	float returnMe = 0.0;
	
	//	rectangle
	if (shape == 0)	{
		if (RotatedPointInTriangle(pt, shapeCoordinates.xy, shapeCoordinates.xy + vec2(0.0, shapeCoordinates.w), shapeCoordinates.xy + vec2(shapeCoordinates.z, 0.0), shapeCoordinates.xy + shapeCoordinates.zw / 2.0))	{
			returnMe = 1.0;
			// soft edge if needed
			if ((pt.x > shapeCoordinates.x) && (pt.x < shapeCoordinates.x)) {
				returnMe = clamp(((pt.x - shapeCoordinates.x) / RENDERSIZE.x), 0.0, 1.0);
				returnMe = pow(returnMe, 0.5);
			}
			else if ((pt.x > shapeCoordinates.x + shapeCoordinates.z) && (pt.x < shapeCoordinates.x + shapeCoordinates.z)) {
				returnMe = clamp(((shapeCoordinates.x + shapeCoordinates.z - pt.x) / RENDERSIZE.x), 0.0, 1.0);
				returnMe = pow(returnMe, 0.5);
			}
		}
		else if (RotatedPointInTriangle(pt, shapeCoordinates.xy + shapeCoordinates.zw, shapeCoordinates.xy + vec2(0.0, shapeCoordinates.w), shapeCoordinates.xy + vec2(shapeCoordinates.z, 0.0), shapeCoordinates.xy + shapeCoordinates.zw / 2.0))	{
			returnMe = 1.0;
			// soft edge if needed
			if ((pt.x > shapeCoordinates.x) && (pt.x < shapeCoordinates.x)) {
				returnMe = clamp(((pt.x - shapeCoordinates.x) / RENDERSIZE.x), 0.0, 1.0);
				returnMe = pow(returnMe, 0.5);
			}
			else if ((pt.x > shapeCoordinates.x + shapeCoordinates.z) && (pt.x < shapeCoordinates.x + shapeCoordinates.z)) {
				returnMe = clamp(((shapeCoordinates.x + shapeCoordinates.z - pt.x) / RENDERSIZE.x), 0.0, 1.0);
				returnMe = pow(returnMe, 0.5);
			}
		}
	}
	//	triangle
	else if (shape == 1)	{
		if (RotatedPointInTriangle(pt, shapeCoordinates.xy, shapeCoordinates.xy + vec2(shapeCoordinates.z / 2.0, shapeCoordinates.w), shapeCoordinates.xy + vec2(shapeCoordinates.z, 0.0), shapeCoordinates.xy + shapeCoordinates.zw / 2.0))	{
			returnMe = 1.0;
		}
	}
	//	oval
	else if (shape == 2)	{
		returnMe = distance(pt, vec2(shapeCoordinates.xy + shapeCoordinates.zw / 2.0));
		if (returnMe < min(shapeCoordinates.z,shapeCoordinates.w) / 2.0)	{
			returnMe = 1.0;
		}
		else	{
			returnMe = 0.0;
		}
	}
	//	diamond
	else if (shape == 3)	{
		if (RotatedPointInTriangle(pt, shapeCoordinates.xy + vec2(0.0, shapeCoordinates.w / 2.0), shapeCoordinates.xy + vec2(shapeCoordinates.z / 2.0, shapeCoordinates.w), shapeCoordinates.xy + vec2(shapeCoordinates.z, shapeCoordinates.w / 2.0), shapeCoordinates.xy + shapeCoordinates.zw / 2.0))	{
			returnMe = 1.0;
		}
		else if (RotatedPointInTriangle(pt, shapeCoordinates.xy + vec2(0.0, shapeCoordinates.w / 2.0), shapeCoordinates.xy + vec2(shapeCoordinates.z / 2.0, 0.0), shapeCoordinates.xy + vec2(shapeCoordinates.z, shapeCoordinates.w / 2.0), shapeCoordinates.xy + shapeCoordinates.zw / 2.0))	{
			returnMe = 1.0;
		}
	}

	return returnMe;	
}

// Value Noise code by Inigo Quilez ported by @colin_movecraft

// Value Noise (http://en.wikipedia.org/wiki/Value_noise), not to be confused with Perlin's
// Noise, is probably the simplest way to generate noise (a random smooth signal with 
// mostly all its energy in the low frequencies) suitable for procedural texturing/shading,
// modeling and animation.
//
// It produces lowe quality noise than Gradient Noise (https://www.shadertoy.com/view/XdXGW8)
// but it is slightly faster to compute. When used in a fractal construction, the blockyness
// of Value Noise gets qcuikly hidden, making it a very popular alternative to Gradient Noise.
//
// The princpiple is to create a virtual grid/latice all over the plane, and assign one
// random value to every vertex in the grid. When querying/requesting a noise value at
// an arbitrary point in the plane, the grid cell in which the query is performed is
// determined (line 30), the four vertices of the grid are determined and their random
// value fetched (lines 35 to 38) and then bilinearly interpolated (lines 35 to 38 again)
// with a smooth interpolant (line 31 and 33).


// Value    Noise 2D, Derivatives: https://www.shadertoy.com/view/4dXBRH
// Gradient Noise 2D, Derivatives: https://www.shadertoy.com/view/XdXBRH
// Value    Noise 3D, Derivatives: https://www.shadertoy.com/view/XsXfRH
// Gradient Noise 3D, Derivatives: https://www.shadertoy.com/view/4dffRH
// Value    Noise 2D             : https://www.shadertoy.com/view/lsf3WH
// Value    Noise 3D             : https://www.shadertoy.com/view/4sfGzS
// Gradient Noise 2D             : https://www.shadertoy.com/view/XdXGW8
// Gradient Noise 3D             : https://www.shadertoy.com/view/Xsl3Dl
// Simplex  Noise 2D             : https://www.shadertoy.com/view/Msf3WH


float hash(vec2 p, float seed)  // replace this by something better
{
    p  = 50.0*fract(seed*1.17921+p*0.3183099 + vec2(0.71,0.113));
    return -1.0+2.0*fract( p.x*p.y*(p.x+p.y) );
}

float noise( in vec2 p , in float seed)
{
    vec2 i = floor( p );
    vec2 f = fract( p );
	
	vec2 u = f*f*(3.0-2.0*f);

    float v1 = mix( mix( hash( i + vec2(0.0,0.0) , floor(seed) ), 
                     hash( i + vec2(1.0,0.0) , floor(seed) ), u.x),
                mix( hash( i + vec2(0.0,1.0) , floor(seed) ), 
                     hash( i + vec2(1.0,1.0) , floor(seed) ), u.x), u.y);
    float v2 = mix( mix( hash( i + vec2(0.0,0.0) , ceil(seed) ), 
                     hash( i + vec2(1.0,0.0) , ceil(seed) ), u.x),
                mix( hash( i + vec2(0.0,1.0) , ceil(seed) ), 
                     hash( i + vec2(1.0,1.0) , ceil(seed) ), u.x), u.y);
    return mix(v1,v2,fract(seed));
}

float map(float n, float i1, float i2, float o1, float o2){
	return o1 + (o2-o1) * (n-i1)/(i2-i1);
	
}

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

vec4 rand4(vec4 co)	{
	vec4	returnMe = vec4(0.0);
	returnMe.r = rand(co.rg);
	returnMe.g = rand(co.gb);
	returnMe.b = rand(co.ba);
	returnMe.a = rand(co.rb);
	return returnMe;
}

vec4 patternForType(vec2 coord, int pType, vec2 seed)	{
	vec4	returnMe = vec4(0.0);
	int		pt = pType;
	if (pt <= 2)	{
    	vec2 p = gl_FragCoord.xy / RENDERSIZE;
		vec2 uv = p*vec2(RENDERSIZE.x/RENDERSIZE.y,1.0);
		float f = 0.0;

		if (glitchSmoothness < 1.0)	{
			float blockness = rand(noiseSeed*(floor(uv*(1.0+glitchScale*50.0))/(1.0+glitchScale*50.0)));
			if (blockness > glitchSmoothness)
				uv = floor(uv*(1.0+glitchScale*20.0))/(1.0+glitchScale*20.0);
		}

		//fbm - fractal noise (4 octaves)	
		{
			uv *= glitchScale;
       	 	mat2 m = mat2( 1.6,  1.2, -1.2,  1.6 );
			f  = 0.5000*noise( uv , patternSeed ); uv = m*uv;
			f += 0.2500*noise( uv , patternSeed ); uv = m*uv;
			f += 0.1250*noise( uv , patternSeed ); uv = m*uv;
			f += 0.0625*noise( uv , patternSeed ); uv = m*uv;
		}
	
		f = pow(f,0.5);
	
		float d = distance(isf_FragNormCoord,vec2(0.5));
		if (d > glitchRadius)	{
			f = f + (d-glitchRadius);
		}
		if (pt == 0)	{
			pt = 5 + int(min(f,1.0) * 3.99);	
		}
		else if (pt == 1)	{
			pt = 3 + int(min(f,1.0) * 3.99);	
		}
		else if (pt == 2)	{
			pt = 4 + int(f * 5.99);
		}
		/*
		if (patternSeed < 0.25)
			pt = 2 + int(4.999 * rand(seed));
		else if (patternSeed < 0.5)
			pt = 2 + int(4.999 * rand(vec2(seed.x*floor(coord.x/(RENDERSIZE.x/4.0)),seed.x*floor(coord.y/(RENDERSIZE.y/4.0)))));
		else if (patternSeed < 0.75)
			pt = 2 + int(4.999 * rand(vec2(seed.x*floor(coord.x/(RENDERSIZE.x/10.0)),seed.x*floor(coord.y/(RENDERSIZE.y/10.0)))));
		else
			pt = 2 + int(4.999 * rand(vec2(seed.x*floor(coord.x/(RENDERSIZE.x/100.0)),seed.x*floor(coord.y/(RENDERSIZE.y/100.0)))));
		*/
	}
	
	vec2	flickCoord = coord;
	if (pt == 5)	{
		flickCoord = vec2(coord.x,floor(coord.y/(1.0+patternSeed))*(1.0+patternSeed));
	}
	else if (pt == 6)	{
		flickCoord = vec2(floor(coord.x/(1.0+patternSeed))*(1.0+patternSeed),coord.y);
	}
	else if (pt > 6) {
		flickCoord = floor(coord/(1.0+patternSeed))*(1.0+patternSeed);
	}
	float	flickRand = (flickerLevel > 0.0) ? rand(TIME * flickCoord + patternSeed) : 0.0;
	bool	doFlick = false;
	if (flickRand < flickerLevel)	{
		doFlick = true;
	}
	//nothing
	if (pt == 3)	{
		returnMe = vec4(0.0);
	}
	//noise
	else if (pt == 4)	{
		if (doFlick)	{
			returnMe = IMG_THIS_PIXEL(inputImage);
		}
		else	{
			vec2	modCoord = RENDERSIZE*vec2(rand(patternSeed * coord),rand(patternSeed * coord.yx));
			modCoord *= max(1.0+patternSeed,1.00001);
			returnMe.r = (mod(modCoord.x+modCoord.y,4.0)<2.0) ? 1.0 : 0.0;
			returnMe.g = (mod(modCoord.x+modCoord.y,5.0)<2.0) ? 1.0 : 0.0;
			returnMe.b = (mod(modCoord.x+modCoord.y,7.0)<2.0) ? 1.0 : 0.0;
			returnMe.a = 1.0;
		}
	}
	//lines
	else if (pt == 5)	{
		vec2	modCoord = (rand(vec2(patternSeed,coord.y)) < 0.25) ? coord : coord.yx;
		modCoord *= (1.0+patternSeed);
		if (doFlick)	{
			returnMe = mix(IMG_PIXEL(inputImage,vec2(modCoord.x,0.0)),IMG_PIXEL(inputImage,vec2(modCoord.x,RENDERSIZE.y)),modCoord.y/RENDERSIZE.y);
		}
		else	{
			returnMe.r = (mod(modCoord.x,4.0)<1.0) ? 1.0 : 0.0;
			returnMe.g = (mod(modCoord.x,5.0)<1.0) ? 1.0 : 0.0;
			returnMe.b = (mod(modCoord.x,7.0)<1.0) ? 1.0 : 0.0;
			returnMe.a = 1.0;
		}
	}
	//vertical lines
	else if (pt == 6)	{
		vec2	modCoord = coord;
		//modCoord.x += rand(vec2(patternSeed,modCoord.x));
		modCoord.x *= (1.0+patternSeed);
		modCoord *= (1.0+patternSeed);
		modCoord = mod(modCoord,RENDERSIZE);
		if (doFlick)	{
			returnMe = mix(IMG_PIXEL(inputImage,vec2(modCoord.x,0.0)),IMG_PIXEL(inputImage,vec2(modCoord.x,RENDERSIZE.y)),modCoord.y/RENDERSIZE.y);
		}
		else	{
			returnMe.r = (mod(modCoord.x,4.0)<1.0) ? 1.0 : 0.0;
			returnMe.g = (mod(modCoord.x,5.0)<1.0) ? 1.0 : 0.0;
			returnMe.b = (mod(modCoord.x,7.0)<1.0) ? 1.0 : 0.0;
			returnMe.a = 1.0;
		}
	}
	//horizontal lines
	else if (pt == 7)	{
		vec2	modCoord = coord;
		//modCoord.y += rand(vec2(patternSeed,modCoord.y));
		modCoord.y *= (1.0+patternSeed);
		modCoord = mod(modCoord,RENDERSIZE);
		if (doFlick)	{
			returnMe = mix(IMG_PIXEL(inputImage,vec2(0.0,modCoord.y)),IMG_PIXEL(inputImage,vec2(RENDERSIZE.x,modCoord.y)),modCoord.x/RENDERSIZE.x);
		}
		else	{
			returnMe.r = (mod(modCoord.y,4.0)<1.0) ? 1.0 : 0.0;
			returnMe.g = (mod(modCoord.y,5.0)<1.0) ? 1.0 : 0.0;
			returnMe.b = (mod(modCoord.y,7.0)<1.0) ? 1.0 : 0.0;
			returnMe.a = 1.0;
		}
	}
	//stripes
	else if (pt == 8)	{
		vec2	modCoord = mod(coord*(1.0+patternSeed),10.0);
		if (doFlick)	{
			returnMe = (modCoord.x<5.0) ? IMG_PIXEL(inputImage,vec2(modCoord.x,0.0)) : IMG_PIXEL(inputImage,vec2(modCoord.x,RENDERSIZE.y));
		}
		else	{
			returnMe = ((modCoord.x<5.0)) ? vec4(1.0) : vec4(0.0,0.0,0.0,1.0);
		}
	}
	//checkerboard
	else 	{
		vec2	modCoord = mod(coord*(1.0+patternSeed),80.0);
		returnMe = vec4(0.9,0.9,0.9,1.0);
		if (doFlick)
			returnMe = IMG_PIXEL(inputImage,floor(coord/(1.0+patternSeed))*(1.0+patternSeed));
		else if ((modCoord.x<40.0)&&(modCoord.y<40.0))
			returnMe = vec4(0.5,0.5,0.5,1.0);
		else if ((modCoord.x>40.0)&&(modCoord.y>40.0))
			returnMe = vec4(0.5,0.5,0.5,1.0);
		
	}
	return returnMe;
}

void main(){
    vec2 p = gl_FragCoord.xy / RENDERSIZE;
    vec4 returnMe = IMG_THIS_PIXEL(inputImage);
    float f = 1.0;
    bool didRowGlitch = false;

	if (rowGlitchLevel > 0.0)	{
		float rowGlitch = rand(vec2(p.y+patternSeed+0.239,noiseSeed+0.821));
		if (rowGlitch < rowGlitchLevel)	{
			rowGlitch = rand(vec2(p.y+1.287*patternSeed+2.239,6.12*noiseSeed+3.477));
			if (p.x < rowGlitch)	{
				returnMe = IMG_NORM_PIXEL(inputImage,vec2(rowGlitch,p.y));
				vec4	bgColor = rand4(vec4(rowGlitch, 1.28*rowGlitchSeed+p.y, rowGlitchSeed+floor(10.0*p.x/rowGlitch), rowGlitchSeed));
				//patternForType(vec2(rowGlitch,p.y)*RENDERSIZE,patternStyle,vec2(noiseSeed,rowGlitchSeed));
				returnMe = mix(returnMe,bgColor,1.0-rowFlickerLevel*rand(vec2(TIME+p.y,rowGlitch+rowGlitchSeed)));
				didRowGlitch = true;
			}
		}
	}
	{
		vec2 uv = p*vec2(RENDERSIZE.x/RENDERSIZE.y,1.0);
		f = 0.0;
		if (glitchSmoothness < 1.0)	{
			float blockness = rand(noiseSeed*(floor(uv*(1.0+glitchScale*100.0))/(1.0+glitchScale*100.0)));
			if (blockness > glitchSmoothness)
				uv = floor(uv*(1.0+glitchScale*25.0))/(1.0+glitchScale*25.0);
		}
		//fbm - fractal noise (4 octaves)	
		{
			uv *= glitchScale;
       		mat2 m = mat2( 1.6,  1.2, -1.2,  1.6 );
			f  = 0.5000*noise( uv , noiseSeed ); uv = m*uv;
			f += 0.2500*noise( uv , noiseSeed ); uv = m*uv;
			f += 0.1250*noise( uv , noiseSeed ); uv = m*uv;
			f += 0.0625*noise( uv , noiseSeed ); uv = m*uv;
		}
	
		f = 1.0-pow(f,(5.0-glitchBrightnessCurve))*glitchBrightness;
	
		float d = distance(isf_FragNormCoord,vec2(0.5));
		if (d > glitchRadius)	{
			f = f + (d-glitchRadius);
		}
	
		f = (f > 1.0) ? 1.0 : f;
	
		
		//vec4 returnMe = vec4(f,f,f,1.0);
		if (hardEdge)	{
			f = (f > 0.9) ? 1.0 : 0.0;	
		}
		if (f < 1.0)	{
			vec4	bgColor = patternForType(gl_FragCoord.xy,patternStyle,vec2(patternSeed,0.2941));
			returnMe = mix(returnMe,bgColor,1.0-f);
		}
	}
	
	{
		vec2 uv = p*vec2(RENDERSIZE.x/RENDERSIZE.y,1.0);
		f = 0.0;
		//fbm - fractal noise (4 octaves)	
		{
			uv *= tintScale;
       		mat2 m = mat2( 1.6,  1.2, -1.2,  1.6 );
			f  = 0.5000*noise( uv , tintSeed ); uv = m*uv;
			f += 0.2500*noise( uv , tintSeed ); uv = m*uv;
			f += 0.1250*noise( uv , tintSeed ); uv = m*uv;
			f += 0.0625*noise( uv , tintSeed ); uv = m*uv;
		}
	
		f = 1.0-pow(f,(5.0-tintBrightnessCurve))*tintBrightness;
	
		float d = distance(isf_FragNormCoord,vec2(0.5));
		if (d > tintRadius)	{
			f = f + (d-tintRadius);
		}
	
		f = (f > 1.0) ? 1.0 : f;
	
		
		//vec4 returnMe = vec4(f,f,f,1.0);
		if (f < 1.0)	{
			vec4	bgColor = mix(tintColor1,tintColor2,f+rand(vec2(tintSeed,0.231)));
			if (hardEdge)	{
				f = (f > 0.9) ? 1.0 : 0.0;	
			}
			returnMe = mix(returnMe,bgColor*returnMe,1.0-f);
		}
	}
	
	if (alphaMode == 1)
		returnMe.a *= f;
	else if (alphaMode == 2)
		returnMe.a = f;
	
	gl_FragColor = returnMe;
}




// The MIT License
// Copyright © 2013 Inigo Quilez
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
