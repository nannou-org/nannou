/*{
    "CATEGORIES": [
        "Color Effect"
    ],
    "CREDIT": "by Carter Rosenberg",
    "DESCRIPTION": "Auto Levels",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "DEFAULT": 0,
            "MAX": 1,
            "MIN": 0,
            "NAME": "min_threshold",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "mid_point",
            "TYPE": "float"
        },
        {
            "DEFAULT": 1,
            "MAX": 1,
            "MIN": 0,
            "NAME": "max_threshold",
            "TYPE": "float"
        },
        {
            "DEFAULT": 0.5,
            "MAX": 1,
            "MIN": 0,
            "NAME": "adapt_rate",
            "TYPE": "float"
        }
    ],
    "ISFVSN": "2",
    "PASSES": [
        {
            "HEIGHT": "$HEIGHT / 2.0",
            "TARGET": "bufferPassA",
            "WIDTH": "$WIDTH / 2.0"
        },
        {
            "HEIGHT": "max($HEIGHT / 27.0, 1.0)",
            "TARGET": "bufferPassB",
            "WIDTH": "max($WIDTH / 27.0, 1.0)"
        },
        {
            "HEIGHT": "max($HEIGHT / 3.0, 1.0)",
            "TARGET": "bufferPassC",
            "WIDTH": "max($WIDTH / 3.0, 1.0)"
        },
        {
            "HEIGHT": "max($HEIGHT / 243.0, 1.0)",
            "TARGET": "bufferPassD",
            "WIDTH": "max($WIDTH / 243.0, 1.0)"
        },
        {
            "HEIGHT": "max($HEIGHT/486.0,1.0)",
            "PERSISTENT": true,
            "TARGET": "bufferVariableNameA",
            "WIDTH": "max($WIDTH/486.0,1.0)"
        },
        {
        }
    ]
}
*/

#if __VERSION__ <= 120
varying vec2 left_coord;
varying vec2 right_coord;
varying vec2 above_coord;
varying vec2 below_coord;

varying vec2 lefta_coord;
varying vec2 righta_coord;
varying vec2 leftb_coord;
varying vec2 rightb_coord;
#else
in vec2 left_coord;
in vec2 right_coord;
in vec2 above_coord;
in vec2 below_coord;

in vec2 lefta_coord;
in vec2 righta_coord;
in vec2 leftb_coord;
in vec2 rightb_coord;
#endif

float gray(vec4 n)
{
	return (n.r + n.g + n.b)/3.0;
}

//	packed as min, mid, max
vec3 minmaxmean(vec4 n, vec3 old)
{
	float val = gray(n);
	vec3 returnMe = old;
	if (val < returnMe.x)	{
		returnMe.x = val;
	}
	if (val > returnMe.z)	{
		returnMe.z = val;
	}
	returnMe.y = returnMe.y + val / 9.0;
	return returnMe;
}

vec3 minmaxcompare(vec4 n, vec3 old)	{
	vec3 returnMe = old;
	if (n.r < returnMe.x)	{
		returnMe.x = n.r;
	}
	if (n.g > returnMe.z)	{
		returnMe.z = n.g;
	}
	returnMe.y = returnMe.y + n.b / 9.0;
	return returnMe;
}


void main()
{
	vec3 vals = vec3(1.0,0.0,0.0);
	
	//	on this first pass we find the local mins, mid, and max for each pixel
	if (PASSINDEX == 0)	{
		vec4 min_color = IMG_THIS_PIXEL(inputImage);
		vec4 min_colorL = IMG_NORM_PIXEL(inputImage, left_coord);
		vec4 min_colorR = IMG_NORM_PIXEL(inputImage, right_coord);
		vec4 min_colorA = IMG_NORM_PIXEL(inputImage, above_coord);
		vec4 min_colorB = IMG_NORM_PIXEL(inputImage, below_coord);
	
		vec4 min_colorLA = IMG_NORM_PIXEL(inputImage, lefta_coord);
		vec4 min_colorRA = IMG_NORM_PIXEL(inputImage, righta_coord);
		vec4 min_colorLB = IMG_NORM_PIXEL(inputImage, leftb_coord);
		vec4 min_colorRB = IMG_NORM_PIXEL(inputImage, rightb_coord);
	
		vals = minmaxmean(min_color,vals);

		vals = minmaxmean(min_colorL,vals);
		vals = minmaxmean(min_colorR,vals);
		vals = minmaxmean(min_colorA,vals);
		vals = minmaxmean(min_colorB,vals);

		vals = minmaxmean(min_colorLA,vals);
		vals = minmaxmean(min_colorRA,vals);
		vals = minmaxmean(min_colorLB,vals);
		vals = minmaxmean(min_colorRB,vals);
		gl_FragColor = vec4(vals.x,vals.y,vals.z,min_color.a);
	}
	//	second pass: read from "bufferPassA", write to "bufferPassB"
	else if (PASSINDEX == 1)	{
		vec4 min_color = IMG_THIS_PIXEL(bufferPassA);
		vec4 min_colorL = IMG_NORM_PIXEL(bufferPassA, left_coord);
		vec4 min_colorR = IMG_NORM_PIXEL(bufferPassA, right_coord);
		vec4 min_colorA = IMG_NORM_PIXEL(bufferPassA, above_coord);
		vec4 min_colorB = IMG_NORM_PIXEL(bufferPassA, below_coord);
	
		vec4 min_colorLA = IMG_NORM_PIXEL(bufferPassA, lefta_coord);
		vec4 min_colorRA = IMG_NORM_PIXEL(bufferPassA, righta_coord);
		vec4 min_colorLB = IMG_NORM_PIXEL(bufferPassA, leftb_coord);
		vec4 min_colorRB = IMG_NORM_PIXEL(bufferPassA, rightb_coord);
	
		vals = minmaxcompare(min_color,vals);

		vals = minmaxcompare(min_colorL,vals);
		vals = minmaxcompare(min_colorR,vals);
		vals = minmaxcompare(min_colorA,vals);
		vals = minmaxcompare(min_colorB,vals);

		vals = minmaxcompare(min_colorLA,vals);
		vals = minmaxcompare(min_colorRA,vals);
		vals = minmaxcompare(min_colorLB,vals);
		vals = minmaxcompare(min_colorRB,vals);
		gl_FragColor = vec4(vals.x,vals.y,vals.z,min_color.a);
	}
	//	read from "bufferPassB", write to "bufferPassC"
	else if (PASSINDEX == 2)	{
		vec4 min_color = IMG_THIS_PIXEL(bufferPassB);
		vec4 min_colorL = IMG_NORM_PIXEL(bufferPassB, left_coord);
		vec4 min_colorR = IMG_NORM_PIXEL(bufferPassB, right_coord);
		vec4 min_colorA = IMG_NORM_PIXEL(bufferPassB, above_coord);
		vec4 min_colorB = IMG_NORM_PIXEL(bufferPassB, below_coord);
	
		vec4 min_colorLA = IMG_NORM_PIXEL(bufferPassB, lefta_coord);
		vec4 min_colorRA = IMG_NORM_PIXEL(bufferPassB, righta_coord);
		vec4 min_colorLB = IMG_NORM_PIXEL(bufferPassB, leftb_coord);
		vec4 min_colorRB = IMG_NORM_PIXEL(bufferPassB, rightb_coord);

		vals = minmaxcompare(min_color,vals);

		vals = minmaxcompare(min_colorL,vals);
		vals = minmaxcompare(min_colorR,vals);
		vals = minmaxcompare(min_colorA,vals);
		vals = minmaxcompare(min_colorB,vals);

		vals = minmaxcompare(min_colorLA,vals);
		vals = minmaxcompare(min_colorRA,vals);
		vals = minmaxcompare(min_colorLB,vals);
		vals = minmaxcompare(min_colorRB,vals);
		gl_FragColor = vec4(vals.x,vals.y,vals.z,min_color.a);
	}
	//	read from "bufferPassC", write to "bufferPassD"
	else if (PASSINDEX == 3)	{
		vec4 min_color = IMG_THIS_PIXEL(bufferPassC);
		vec4 min_colorL = IMG_NORM_PIXEL(bufferPassC, left_coord);
		vec4 min_colorR = IMG_NORM_PIXEL(bufferPassC, right_coord);
		vec4 min_colorA = IMG_NORM_PIXEL(bufferPassC, above_coord);
		vec4 min_colorB = IMG_NORM_PIXEL(bufferPassC, below_coord);
	
		vec4 min_colorLA = IMG_NORM_PIXEL(bufferPassC, lefta_coord);
		vec4 min_colorRA = IMG_NORM_PIXEL(bufferPassC, righta_coord);
		vec4 min_colorLB = IMG_NORM_PIXEL(bufferPassC, leftb_coord);
		vec4 min_colorRB = IMG_NORM_PIXEL(bufferPassC, rightb_coord);
	
		vals = minmaxcompare(min_color,vals);

		vals = minmaxcompare(min_colorL,vals);
		vals = minmaxcompare(min_colorR,vals);
		vals = minmaxcompare(min_colorA,vals);
		vals = minmaxcompare(min_colorB,vals);

		vals = minmaxcompare(min_colorLA,vals);
		vals = minmaxcompare(min_colorRA,vals);
		vals = minmaxcompare(min_colorLB,vals);
		vals = minmaxcompare(min_colorRB,vals);
		gl_FragColor = vec4(vals.x,vals.y,vals.z,min_color.a);
	}
	//	read from "bufferPassD", write to "bufferVariableNameA"
	else if (PASSINDEX == 4)	{
		vec4 min_color = IMG_THIS_PIXEL(bufferPassD);
		vec4 stalePixel = IMG_THIS_PIXEL(bufferVariableNameA);
		vec4 min_colorL = IMG_NORM_PIXEL(bufferPassD, left_coord);
		vec4 min_colorR = IMG_NORM_PIXEL(bufferPassD, right_coord);
		vec4 min_colorA = IMG_NORM_PIXEL(bufferPassD, above_coord);
		vec4 min_colorB = IMG_NORM_PIXEL(bufferPassD, below_coord);
	
		vec4 min_colorLA = IMG_NORM_PIXEL(bufferPassD, lefta_coord);
		vec4 min_colorRA = IMG_NORM_PIXEL(bufferPassD, righta_coord);
		vec4 min_colorLB = IMG_NORM_PIXEL(bufferPassD, leftb_coord);
		vec4 min_colorRB = IMG_NORM_PIXEL(bufferPassD, rightb_coord);
	
		vals = minmaxcompare(min_color,vals);

		vals = minmaxcompare(min_colorL,vals);
		vals = minmaxcompare(min_colorR,vals);
		vals = minmaxcompare(min_colorA,vals);
		vals = minmaxcompare(min_colorB,vals);

		vals = minmaxcompare(min_colorLA,vals);
		vals = minmaxcompare(min_colorRA,vals);
		vals = minmaxcompare(min_colorLB,vals);
		vals = minmaxcompare(min_colorRB,vals);
		gl_FragColor = mix(vec4(vals.x,vals.y,vals.z,min_color.a),stalePixel,adapt_rate);;
	}
	//	read from "bufferVariableNameA", write to output
	else if (PASSINDEX == 5)	{
		vec4 min_color = IMG_THIS_PIXEL(bufferVariableNameA);
		vec4 min_colorL = IMG_NORM_PIXEL(bufferVariableNameA, left_coord);
		vec4 min_colorR = IMG_NORM_PIXEL(bufferVariableNameA, right_coord);
		vec4 min_colorA = IMG_NORM_PIXEL(bufferVariableNameA, above_coord);
		vec4 min_colorB = IMG_NORM_PIXEL(bufferVariableNameA, below_coord);
	
		vec4 min_colorLA = IMG_NORM_PIXEL(bufferVariableNameA, lefta_coord);
		vec4 min_colorRA = IMG_NORM_PIXEL(bufferVariableNameA, righta_coord);
		vec4 min_colorLB = IMG_NORM_PIXEL(bufferVariableNameA, leftb_coord);
		vec4 min_colorRB = IMG_NORM_PIXEL(bufferVariableNameA, rightb_coord);
	
		vals = minmaxcompare(min_color,vals);

		vals = minmaxcompare(min_colorL,vals);
		vals = minmaxcompare(min_colorR,vals);
		vals = minmaxcompare(min_colorA,vals);
		vals = minmaxcompare(min_colorB,vals);

		vals = minmaxcompare(min_colorLA,vals);
		vals = minmaxcompare(min_colorRA,vals);
		vals = minmaxcompare(min_colorLB,vals);
		vals = minmaxcompare(min_colorRB,vals);
		
		vec4 final_pixel = IMG_THIS_PIXEL(inputImage);
		
		
		float val = gray (final_pixel);
		if (vals.x < min_threshold)	{
			vals.x = min_threshold;
		}
		if (val < min_threshold)	{
			val = min_threshold;
		}
		if (vals.z > max_threshold)	{
			vals.z = max_threshold;
		}
		if (val > max_threshold)	{
			val = max_threshold;
		}
		//final_pixel.rgb = (final_pixel.rgb - vals.x) / (vals.z - vals.x);
		//	shift by the absdiff of the midpoint
		//final_pixel.rgb = final_pixel.rgb + (mid_point - vals.y);
		final_pixel.rgb = final_pixel.rgb - (0.5 - mid_point);
		//	by contrast adjustments
		float diff = vals.z-vals.x;
		if (diff < 0.01)	{
			diff = 0.01;
		}
		final_pixel.rgb = (final_pixel.rgb - vals.x) / diff;
		//float contrast = 1.0/diff;		
		//final_pixel.rgb = ((vec3(2.0) * (final_pixel.rgb - vec3(vals.x))) * vec3(contrast) / vec3(2.0)) + vec3(0.5);
		
		gl_FragColor = final_pixel;
		//gl_FragColor = vec4(vec3(vals.z),1.0);
	}
}
