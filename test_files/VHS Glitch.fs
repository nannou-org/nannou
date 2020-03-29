/*{
    "CATEGORIES": [
        "Glitch",
        "Retro"
    ],
    "CREDIT": "David Lublin, original by Staffan Widegarn Åhlvik",
    "DESCRIPTION": "VHS Glitch Style",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 1,
            "NAME": "autoScan",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "xScanline",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "xScanline2",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "yScanline",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "xScanlineSize",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": 0,
            "NAME": "xScanlineSize2",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.25,
            "MAX": 1,
            "MIN": -1,
            "NAME": "yScanlineAmount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0,
            "MAX": 3,
            "MIN": 0,
            "NAME": "grainLevel",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "NAME": "scanFollow",
            "TYPE": "bool"
        },
        {
            "DEFAULT": 1,
            "MAX": 10,
            "MIN": 0,
            "NAME": "analogDistort",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 3,
            "MIN": 0,
            "NAME": "bleedAmount",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "bleedDistort",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 2,
            "MIN": 0,
            "NAME": "bleedRange",
            "TYPE": "float"
        },
        {
            "DEFAULT": [
                0.8,
                0,
                0.4,
                1
            ],
            "NAME": "colorBleedL",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0,
                0.5,
                0.75,
                1
            ],
            "NAME": "colorBleedC",
            "TYPE": "color"
        },
        {
            "DEFAULT": [
                0.8,
                0,
                0.4,
                1
            ],
            "NAME": "colorBleedR",
            "TYPE": "color"
        }
    ],
    "ISFVSN": "2"
}
*/




//	Based on https://github.com/staffantan/unity-vhsglitch
//	Converted by David Lublin / VIDVOX


const float tau = 6.28318530718;


float rand(vec3 co){
	return abs(mod(sin( dot(co.xyz ,vec3(12.9898,78.233,45.5432) )) * 43758.5453, 1.0));
}

void main()	{
	float	actualXLine = (!autoScan) ? xScanline : mod(xScanline + ((1.0+sin(0.34*TIME))/2.0 + (1.0+sin(TIME))/3.0 + (1.0+cos(2.1*TIME))/3.0 + (1.0+cos(0.027*TIME))/2.0)/3.5,1.0);
	float	actualXLineWidth = (!autoScan) ? xScanlineSize : xScanlineSize + ((1.0+sin(1.2*TIME))/2.0 + (1.0+cos(3.91*TIME))/3.0 + (1.0+cos(0.014*TIME))/2.0)/3.5;
	vec2	loc = isf_FragNormCoord;
	vec4	vhs = IMG_NORM_PIXEL(inputImage, loc);
	float	dx = 1.0+actualXLineWidth/25.0-abs(distance(loc.y, actualXLine));
	float	dx2 = 1.0+xScanlineSize2/10.0-abs(distance(loc.y, xScanline2));
	float	dy = (1.0-abs(distance(loc.y, yScanline)));
	if (autoScan)
		dy = (1.0-abs(distance(loc.y, mod(yScanline+TIME,1.0))));
	
	dy = (dy > 0.5) ? 2.0 * dy : 2.0 * (1.0 - dy);
	
	float	rX = (scanFollow) ? rand(vec3(dy,actualXLine,analogDistort)) : rand(vec3(dy,bleedAmount,analogDistort));
	float	xTime = (actualXLine > 0.5) ? 2.0 * actualXLine : 2.0 * (1.0 - actualXLine);
	
	loc.x += yScanlineAmount * dy * 0.025 + analogDistort * rX/(RENDERSIZE.x/2.0);
	
	if(dx2 > 1.0 - xScanlineSize2 / 10.0)	{
		float	rX2 = (dy * rand(vec3(dy,dx2,dx+TIME)) + dx2) / 4.0;
		float	distortAmount = analogDistort * (sin(rX * tau / dx2) + cos(rX * tau * 0.78 / dx2)) / 10.0;
		//loc.y = xScanline2;
		//loc.x += (1.0 + distortAmount * sin(tau * (loc.x) / rX2 ) - 1.0) / 15.0;
		loc.x += (1.0 + distortAmount * sin(tau * (loc.x) / rX2 ) - 1.0) / 15.0;
	}
	if(dx > 1.0 - actualXLineWidth / 25.0)
		loc.y = actualXLine;

	loc.x = mod(loc.x,1.0);
	loc.y = mod(loc.y,1.0);
	
	vec4	c = IMG_NORM_PIXEL(inputImage, loc);
	float	x = (loc.x*320.0)/320.0;
	float	y = (loc.y*240.0)/240.0;
	float	bleed = 0.0;
	
	if (scanFollow)
		c -= rand(vec3(x, y, xTime)) * xTime / (5.0-grainLevel);
	else
		c -= rand(vec3(x, y, bleedAmount)) * (bleedAmount/20.0) / (5.0-grainLevel);
	
	if (bleedAmount > 0.0)	{
		IMG_NORM_PIXEL(inputImage, loc + vec2(0.01, 0)).r;
		bleed += IMG_NORM_PIXEL(inputImage, loc + bleedRange * vec2(0.02, 0)).r;
		bleed += IMG_NORM_PIXEL(inputImage, loc + bleedRange * vec2(0.01, 0.01)).r;
		bleed += IMG_NORM_PIXEL(inputImage, loc + bleedRange * vec2(-0.02, 0.02)).r;
		bleed += IMG_NORM_PIXEL(inputImage, loc + bleedRange * vec2(0.0, -0.03)).r;
		bleed /= 6.0;
		bleed *= bleedAmount;
	}

	if (bleed > 0.1){
		float	bleedFreq = 1.0;
		float	bleedX = 0.0;
		if (autoScan)
			bleedX = x + bleedDistort * (yScanlineAmount + (1.5 + cos(TIME / 13.0 + tau*(bleedDistort+(1.0-loc.y))))/2.0) * sin((TIME / 9.0 + bleedDistort) * tau + loc.y * loc.y * tau * bleedFreq) / 8.0;
		else
			bleedX = x + (yScanlineAmount + (1.0 + sin(tau*(bleedDistort+loc.y)))/2.0) * sin(bleedDistort * tau + loc.y * loc.y * tau * bleedFreq) / 10.0;
		vec4	colorBleed = (bleedX < 0.5) ? mix(colorBleedL, colorBleedC, 2.0 * bleedX) : mix(colorBleedR, colorBleedC, 2.0 - 2.0 * bleedX);
		if (scanFollow)
			c += bleed * max(xScanlineSize,xTime) * colorBleed;
		else
			c += bleed * colorBleed;
	}
	gl_FragColor = c;
}
