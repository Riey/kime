#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Bgra(pub [u8; 4]);

impl image::Pixel for Bgra {
    type Subpixel = u8;

    const CHANNEL_COUNT: u8 = 4;

    fn channels(&self) -> &[Self::Subpixel] {
        self.0.as_slice()
    }

    fn channels_mut(&mut self) -> &mut [Self::Subpixel] {
        self.0.as_mut_slice()
    }

    const COLOR_MODEL: &'static str = "BGRA";

    fn channels4(
        &self,
    ) -> (
        Self::Subpixel,
        Self::Subpixel,
        Self::Subpixel,
        Self::Subpixel,
    ) {
        (self.0[0], self.0[1], self.0[2], self.0[3])
    }

    fn from_channels(
        a: Self::Subpixel,
        b: Self::Subpixel,
        c: Self::Subpixel,
        d: Self::Subpixel,
    ) -> Self {
        Self([a, b, c, d])
    }

    fn from_slice(slice: &[Self::Subpixel]) -> &Self {
        unsafe { slice.as_ptr().cast::<Self>().as_ref().unwrap_unchecked() }
    }

    fn from_slice_mut(slice: &mut [Self::Subpixel]) -> &mut Self {
        unsafe {
            slice
                .as_mut_ptr()
                .cast::<Self>()
                .as_mut()
                .unwrap_unchecked()
        }
    }

    fn to_rgb(&self) -> image::Rgb<Self::Subpixel> {
        image::Rgb([self.0[2], self.0[1], self.0[0]])
    }

    fn to_rgba(&self) -> image::Rgba<Self::Subpixel> {
        image::Rgba([self.0[2], self.0[1], self.0[0], self.0[3]])
    }

    fn to_luma(&self) -> image::Luma<Self::Subpixel> {
        self.to_rgb().to_luma()
    }

    fn to_luma_alpha(&self) -> image::LumaA<Self::Subpixel> {
        self.to_rgba().to_luma_alpha()
    }

    fn map<F>(&self, f: F) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        Self(self.0.map(f))
    }

    fn apply<F>(&mut self, f: F)
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        self.0 = self.0.map(f);
    }

    fn map_with_alpha<F, G>(&self, f: F, g: G) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
        G: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        let mut ret = self.clone();
        ret.apply_with_alpha(f, g);
        ret
    }

    fn apply_with_alpha<F, G>(&mut self, mut f: F, mut g: G)
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
        G: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        for c in &mut self.0[..3] {
            *c = f(*c);
        }
        self.0[3] = g(self.0[3]);
    }

    fn map2<F>(&self, other: &Self, f: F) -> Self
    where
        F: FnMut(Self::Subpixel, Self::Subpixel) -> Self::Subpixel,
    {
        let mut ret = self.clone();
        ret.apply2(other, f);
        ret
    }

    fn apply2<F>(&mut self, other: &Self, mut f: F)
    where
        F: FnMut(Self::Subpixel, Self::Subpixel) -> Self::Subpixel,
    {
        for (a, b) in self.0.iter_mut().zip(other.0.iter()) {
            *a = f(*a, *b);
        }
    }

    fn invert(&mut self) {
        self.apply_without_alpha(|c| u8::MAX - c);
    }

    fn blend(&mut self, other: &Self) {
        let mut rgba = self.to_rgba();
        rgba.blend(&other.to_rgba());
        let [r, g, b, a] = rgba.0;
        self.0 = [b, g, r, a];
    }
}
