/*
{
  "CATEGORIES" : [
    "Blur",
    "Glitch"
  ],
  "DESCRIPTION" : "This converts to YIQ and does separate feedback blur on the Y and C channels",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "yFeedbackLevel",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0,
      "MIN" : 0
    },
    {
      "NAME" : "cFeedbackLevel",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0,
      "MIN" : 0
    }
  ],
  "PASSES" : [
    {
      "TARGET" : "lastFrame",
      "PERSISTENT" : true,
      "FLOAT" : true
    }
  ],
  "CREDIT" : "VIDVOX"
}
*/


//	inspired by (but not exactly the same thing as) y/c delay error
//	https://bavc.github.io/avaa/artifacts/yc_delay_error.html

//	shout out to this stackoverflow
//	https://stackoverflow.com/questions/9234724/how-to-change-hue-of-a-texture-with-glsl/9234854#9234854
//	(though I'd have gone to HSV for that particular effect, it covers our use case of yiq nicely!)


const vec4  kRGBToYPrime = vec4 (0.299, 0.587, 0.114, 0.0);
const vec4  kRGBToI     = vec4 (0.596, -0.275, -0.321, 0.0);
const vec4  kRGBToQ     = vec4 (0.212, -0.523, 0.311, 0.0);

const vec4  kYIQToR   = vec4 (1.0, 0.956, 0.621, 0.0);
const vec4  kYIQToG   = vec4 (1.0, -0.272, -0.647, 0.0);
const vec4  kYIQToB   = vec4 (1.0, -1.107, 1.704, 0.0);


vec3 rgb2yiq(vec3 c)	{
    float   YPrime = dot (c, kRGBToYPrime.rgb);
    float   I = dot (c, kRGBToI.rgb);
    float   Q = dot (c, kRGBToQ.rgb);
    return vec3(YPrime,I,Q);
}

vec3 yiq2rgb(vec3 c)	{
    vec3    yIQ   = vec3 (c.r, c.g, c.b);
    return vec3(dot (yIQ, kYIQToR.rgb),dot (yIQ, kYIQToG.rgb),dot (yIQ, kYIQToB.rgb));
}


void main()	{
	//	this is a pass-thru1
	vec4		inputPixelColor = IMG_THIS_PIXEL(inputImage);
	vec4		lastPixel = IMG_THIS_PIXEL(lastFrame);
	inputPixelColor.rgb = rgb2yiq(inputPixelColor.rgb);
	lastPixel.rgb = rgb2yiq(lastPixel.rgb);
	
	inputPixelColor.r = mix(inputPixelColor.r,lastPixel.r,yFeedbackLevel);
	inputPixelColor.gb = mix(inputPixelColor.gb,lastPixel.gb,cFeedbackLevel);
	
	inputPixelColor.rgb = yiq2rgb(inputPixelColor.rgb);
	
	gl_FragColor = inputPixelColor;
}
