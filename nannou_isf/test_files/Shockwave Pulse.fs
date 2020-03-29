/*
{
  "CATEGORIES" : [
    "Distortion Effect"
  ],
  "DESCRIPTION" : "",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    },
    {
      "NAME" : "pulse",
      "TYPE" : "event"
    },
    {
      "NAME" : "rate",
      "TYPE" : "float",
      "MAX" : 4,
      "DEFAULT" : 1,
      "LABEL" : "rate",
      "MIN" : 0
    },
    {
      "NAME" : "magnitude",
      "TYPE" : "float",
      "MAX" : 0.20000000000000001,
      "DEFAULT" : 0.080000000000000002,
      "LABEL" : "magnitude",
      "MIN" : 0
    },
    {
      "NAME" : "distortion",
      "TYPE" : "float",
      "MAX" : 20,
      "DEFAULT" : 10,
      "LABEL" : "distortion",
      "MIN" : 0
    },
    {
      "NAME" : "center",
      "TYPE" : "point2D",
      "MAX" : [
        1,
        1
      ],
      "DEFAULT" : [
        0.5,
        0.5
      ],
      "MIN" : [
        0,
        0
      ]
    }
  ],
  "PASSES" : [
    {
      "PERSISTENT" : true,
      "WIDTH" : "1",
      "DESCRIPTION" : "this buffer stores the last frame's time offset in the first component of its only pixel- note that it's requesting a FLOAT target buffer...",
      "HEIGHT" : "1",
      "TARGET" : "lastTime",
      "FLOAT" : true
    },
    {

    }
  ],
  "CREDIT" : "by VIDVOX"
}
*/



void main()
{
	//	if this is the first pass, i'm going to read the position from the "lastPosition" image, and write a new position based on this and the hold variables
	if (PASSINDEX == 0)	{
		vec4		srcPixel = IMG_PIXEL(lastTime,vec2(0.5));
		//	i'm only using the X, which is the last render time we reset
		srcPixel.r = (pulse && FRAMEINDEX>10) ? 0.0 : clamp(srcPixel.r + rate * 0.01,0.0,1.0);
		gl_FragColor = srcPixel;
	}
	//	else this isn't the first pass- read the position value from the buffer which stores it
	else	{
		vec2 uv = isf_FragNormCoord.xy;
		vec2 texCoord = uv;
		vec2 mod_center = center;
		float distance = distance(uv, mod_center);
		vec4 lastPosVector = IMG_PIXEL(lastTime,vec2(0.5));
		float adustedTime = lastPosVector.r * (1.0+length(distance));

		if ( (distance <= (adustedTime + magnitude)) && (distance >= (adustedTime - magnitude)) ) 	{
			float diff = (distance - adustedTime); 
			float powDiff = 1.0 - pow(abs(diff*distortion), 
									0.8); 
			float diffTime = diff  * powDiff; 
			vec2 diffUV = normalize(uv - mod_center); 
			texCoord = uv + (diffUV * diffTime);
		} 
		gl_FragColor = IMG_NORM_PIXEL(inputImage, texCoord);
	}
}
