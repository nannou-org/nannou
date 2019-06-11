//use nannou::prelude::*;
use nannou::color::pixel::Srgb;
use nannou::color::Mix;
use nannou::color::Rgb;
use nannou::color::Hsv;

pub struct Palette {
    pub colors: Vec<Rgb>,
    pub len: usize,
}

impl Palette {
    pub fn new() -> Self {
        //anime sky
        let colors: [u32; 49] = [0xFF15283D,0xFF0F1925,0xFF203D59,0xFF2E2A33,0xFF3B4259,0xFF487EB3,0xFF4F537E,0xFF325C83,0xFF5A5366,0xFF5696C3,0xFF2D3A68,0xFF71729D,0xFF4C344D,0xFF6B5457,0xFF785272,0xFF7B697E,0xFF472429,0xFF43649F,0xFF682D44,0xFF61AEE9,0xFF9387AA,0xFF9D4A60,0xFF822E37,0xFFB98377,0xFF87A0D1,0xFFAA6E81,0xFFC5737A,0xFFB69EB0,0xFF8D5658,0xFF907070,0xFFD69D9E,0xFFF5BC9F,0xFFB87BA0,0xFFFFFCE1,0xFFFCDCC5,0xFF73D3F6,0xFFE287A3,0xFFDA4945,0xFFF19888,0xFFFDD89E,0xFFEAC2BE,0xFFFEF3C6,0xFFD89A76,0xFFD8616A,0xFFF6B873,0xFFB4594E,0xFFF17F63,0xFFE0E1EA,0xFFA4A9A5];
        let colorsv = colors.to_vec();

        //do the conversion myself
        let mut colsrgb : Vec<Rgb> = colorsv.into_iter().map(|c| {
            let blue: u8 = (c & 0xFF) as u8;
            let green: u8 = (( c >> 8 ) & 0xFF) as u8;
            let red: u8 = (( c >> 16 ) & 0xFF) as u8;
            let c = Srgb::new_u8(red, green,blue);
            c.into()
        }).collect();
    
        //sort on sat/value/hue
        colsrgb.sort_unstable_by(|&a, &b| {
            let ahsv:Hsv = a.into();
            let bhsv:Hsv = b.into();
            //colors are rgb
            //convert to hsv
            let ahue = ahsv.hue.to_positive_radians();
            let bhue = bhsv.hue.to_positive_radians();
            ahue.partial_cmp(&bhue).unwrap()
        });
        
        let len = colors.len();
        Palette {colors : colsrgb, len: len}
    }

    pub fn somecolor_mod(&self, index:usize) -> Rgb {
        self.colors[index % self.colors.len()]
    }
    pub fn somecolor_frac(&self, mut frac:f32) -> Rgb {
        while frac < 0.0 {
            frac += 1.0;
        }
        while frac >= 1.0 {
            frac -= 1.0;
        }
        
        let index = (frac*self.colors.len() as f32 ) as usize;
        self.colors[index]
    }
    pub fn somecolor_frac_interpol(&self, mut frac:f32) -> Rgb {
        while frac < 0.0 { frac += 1.0; }
        while frac >= 1.0 { frac -= 1.0; }
        let index = (frac*(self.colors.len()) as f32 ).floor() as usize;
        let mut index2 = index+1;
        if index2 > self.colors.len()-1 {
            index2 = 0
        }
        let frac_remainder = (frac*(self.colors.len()) as f32)-(index as f32);
        let a = self.colors[index];
        let b = self.colors[index2];
        a.mix(&b,frac_remainder)
    }
}
