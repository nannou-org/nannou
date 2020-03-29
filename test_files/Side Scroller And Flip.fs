/*{
 "CREDIT": "BrianChasalow",
 "ISFVSN": "2",
 "CATEGORIES": [
 "Geometry Adjustment"
 ],
 "INPUTS": [
 {
 "NAME": "inputImage",
 "TYPE": "image"
 },
 {
 "NAME": "slide",
 "TYPE": "float",
 "MIN": 0.0,
 "MAX": 2.0,
 "DEFAULT": 0.0
 },
 {
 "NAME": "shift",
 "TYPE": "float",
 "MIN": 0.0,
 "MAX": 2.0,
 "DEFAULT": 0.0
 },
 {
 "NAME": "mirrorHorizontal",
 "TYPE": "bool",
 "MIN": false,
 "MAX": true,
 "DEFAULT": true
 },
 {
 "NAME": "mirrorVertical",
 "TYPE": "bool",
 "MIN": false,
 "MAX": true,
 "DEFAULT": true
 }
 
 ]
 }*/

void main(void)
{
	vec2 retard = isf_FragNormCoord;
	retard.x += slide;
	retard.y += shift;
	vec2 moddedRetard = mod(retard,1.0);
	
	if(mirrorHorizontal && retard.x >= 1.0 && retard.x <= 2.0)
		moddedRetard = vec2(1.0-moddedRetard.x, moddedRetard.y);
	if(mirrorVertical && retard.y >= 1.0 && retard.y <= 2.0)
		moddedRetard = vec2(moddedRetard.x, 1.0-moddedRetard.y);
	
	vec4 pixel = IMG_NORM_PIXEL(inputImage, moddedRetard);
	gl_FragColor = pixel;
}