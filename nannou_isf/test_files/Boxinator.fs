/*{
	"CREDIT": "by mojovideotech",
	"ISFVSN": "2",
	"CATEGORIES": [
		"Stylize"
	],
	"INPUTS": [
		{
			"NAME": "inputImage",
			"TYPE": "image"
		},
		{
			"NAME": "rate",
			"TYPE": "float",
			"DEFAULT": 2.5,
			"MIN": 0.0,
			"MAX": 10.0
		},
		{
			"NAME": "edge",
			"TYPE": "float",
			"DEFAULT": 0.001,
			"MIN": 0.0,
			"MAX": 0.01
		},
		{
			"NAME": "blend",
			"TYPE": "float",
			"DEFAULT": 0.95,
			"MIN": -1.0,
			"MAX": 1.0
		},
		{
			"NAME": "randomize",
			"TYPE": "float",
			"DEFAULT": 0.5,
			"MIN": 0.0,
			"MAX": 1.0
		},
		{
			"NAME": "gamma",
			"TYPE": "float",
			"DEFAULT": -0.3,
			"MIN": -0.5,
			"MAX": 0.2
		},
		{
			"NAME": "grid",
			"TYPE": "point2D",
			"DEFAULT": [ 64.0, 36.0 ],
			"MIN": [ 1.5, 1.5 ],
			"MAX": [ 900.0, 600.0 ]
		}
	]
}*/

////////////////////////////////////////////////////////////////////
// Boxinator  by mojovideotech
//
// License Creative Commons Attribution-NonCommercial-ShareAlike 3.0
////////////////////////////////////////////////////////////////////


#ifdef GL_ES
precision mediump float;
#endif


//------------------------------------------------------------------
// simplex noise function
// by : Ian McEwan, Ashima Arts
// © 2011 Ashima Arts,  MIT License

vec4 permute(vec4 x) { return mod(((x*34.0)+1.0)*x, 289.0); }

vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314 * r; }

float snoise(vec3 v) { 
  const vec2  C = vec2(1.0/6.0, 1.0/3.0) ;
  const vec4  D = vec4(0.0, 0.5, 1.0, 2.0);
  vec3 i  = floor(v + dot(v, C.yyy) );
  vec3 x0 =   v - i + dot(i, C.xxx) ;
  vec3 g = step(x0.yzx, x0.xyz);
  vec3 l = 1.0 - g;
  vec3 i1 = min( g.xyz, l.zxy );
  vec3 i2 = max( g.xyz, l.zxy );
  vec3 x1 = x0 - i1 + 1.0 * C.xxx;
  vec3 x2 = x0 - i2 + 2.0 * C.xxx;
  vec3 x3 = x0 - 1. + 3.0 * C.xxx;
  i = mod(i, 289.0 ); 
  vec4 p = permute( permute( permute( 
             i.z + vec4(0.0, i1.z, i2.z, 1.0 ))
           + i.y + vec4(0.0, i1.y, i2.y, 1.0 )) 
           + i.x + vec4(0.0, i1.x, i2.x, 1.0 ));
  float n_ = 1.0/7.0; // N=7
  vec3  ns = n_ * D.wyz - D.xzx;
  vec4 j = p - 49.0 * floor(p * ns.z *ns.z);  //  mod(p,N*N)
  vec4 x_ = floor(j * ns.z);
  vec4 y_ = floor(j - 7.0 * x_ );    // mod(j,N)
  vec4 x = x_ *ns.x + ns.yyyy;
  vec4 y = y_ *ns.x + ns.yyyy;
  vec4 h = 1.0 - abs(x) - abs(y);
  vec4 b0 = vec4( x.xy, y.xy );
  vec4 b1 = vec4( x.zw, y.zw );
  vec4 s0 = floor(b0)*2.0 + 1.0;
  vec4 s1 = floor(b1)*2.0 + 1.0;
  vec4 sh = -step(h, vec4(0.0));
  vec4 a0 = b0.xzyw + s0.xzyw*sh.xxyy ;
  vec4 a1 = b1.xzyw + s1.xzyw*sh.zzww ;
  vec3 p0 = vec3(a0.xy,h.x);
  vec3 p1 = vec3(a0.zw,h.y);
  vec3 p2 = vec3(a1.xy,h.z);
  vec3 p3 = vec3(a1.zw,h.w);
  vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2, p2), dot(p3,p3)));
  p0 *= norm.x;
  p1 *= norm.y;
  p2 *= norm.z;
  p3 *= norm.w;
  vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
  m = m * m;
  return 42.0 * dot( m*m, vec4( dot(p0,x0), dot(p1,x1), 
                                dot(p2,x2), dot(p3,x3) ) );
}
//------------------------------------------------------------------

float hash(float h) { return fract(sin(h) * 43758.5453123); }

vec2 tile(vec2 cell, vec2 size) { return fract(cell*size); }

float box(vec2 a, vec2 b){ vec2 o = step(b,a); return o.x*o.y; }

void main(void){
	float T = TIME*rate;
    vec2 uv = gl_FragCoord.xy/RENDERSIZE.xy;
    vec2 g = floor(grid.xy);
    float C = g.x*g.y ;                                                                                                                                          
    float I = 1.0 + floor(uv.x * g.x) + g.y * floor(uv.y * g.y) + g.x;
	vec2 st = tile(uv, g);
    float S = I / C * box(st, vec2(edge*g.xy));
    S = mix(S,hash(S),randomize);
    vec3 color = vec3(S*T);
	float n = snoise(color+IMG_NORM_PIXEL(inputImage, uv.xy).xyz*blend);  

    gl_FragColor = sqrt(max(vec4(vec3(n, n, n ),1.0)+IMG_NORM_PIXEL(inputImage, uv.xy),0.0)+gamma);
}

