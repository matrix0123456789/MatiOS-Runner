
#[repr(C)]
pub struct Bitmap {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
    pub pixel_ratio: f32,
}

#[repr(C)]
#[derive(Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub fn to32bitInt(&self) -> u32 {
        let mut r = self.r;
        if (r < 0.0) {
            r = 0.0;
        } else if r > 1.0 {
            r = 1.0;
        }
        let mut g=self.g;
        if(g<0.0){
            g=0.0;
        }else if(g>1.0){
            g=1.0;
        }
        let mut b=self.b;
        if(b>1.0){
            b=1.0
        }else if(b<0.0){
            b=0.0
        }
        let mut a=self.a;
        if(a>1.0){
            a=1.0
        }else if(a<0.0){
            a=0.0
        }

        return (b * 255.0) as u32
            | ((g * 255.0) as u32) << 8
            | ((r * 255.0) as u32) << 16
            | ((a * 255.0) as u32) << 24;
    }
}
