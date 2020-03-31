/*
{
  "CATEGORIES" : [
    "Pattern", "Color"
  ],
  "DESCRIPTION" : "Generates a basic brick pattern",
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "brickSize",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.2,
      "MIN" : 0
    },
    {
      "NAME" : "fillSize",
      "TYPE" : "float",
      "MAX" : 1,
      "DEFAULT" : 0.9,
      "MIN" : 0
    },
    {
      "NAME" : "brickOffset",
      "TYPE" : "point2D",
      "MAX" : [
        1,
        1
      ],
      "DEFAULT" : [
        0,
        0
      ],
      "MIN" : [
        0,
        0
      ]
    },
    {
      "NAME" : "fillColor",
      "TYPE" : "color",
      "DEFAULT" : [
        0,
        0,
        0,
        0
      ]
    },
    {
      "NAME" : "brickColor",
      "TYPE" : "color",
      "DEFAULT" : [
        1,
        1,
        1,
        1
      ]
    }
  ],
  "CREDIT" : "patriciogv"
}
*/

// Based on Brick example from https://thebookofshaders.com/09/
// Author @patriciogv ( patriciogonzalezvivo.com ) - 2015



vec2 brickTile(vec2 _st, float _zoom){
    _st *= _zoom;

    // Here is where the offset is happening
    _st.x += step(1., mod(_st.y,2.0)) * 0.5;

    return fract(_st);
}

float box(vec2 _st, vec2 _size){
    _size = vec2(0.5)-_size*0.5;
    vec2 uv = smoothstep(_size,_size+vec2(1e-4),_st);
    uv *= smoothstep(_size,_size+vec2(1e-4),vec2(1.0)-_st);
    return uv.x*uv.y;
}

void main(void){
    vec2 st = isf_FragNormCoord;
    vec4 color = fillColor;
    float brickCount = (brickSize == 0.0) ? max(RENDERSIZE.x,RENDERSIZE.y) : 1.0 / brickSize;
    //	Apply the offset
    st += brickOffset;
    // Apply the brick tiling
    st = brickTile(st,brickCount);

	float val = box(st,vec2(fillSize));
    color = mix(fillColor,brickColor,val);

    // Uncomment to see the space coordinates
    //color.rgb = vec3(st,0.0);

    gl_FragColor = color;
}

