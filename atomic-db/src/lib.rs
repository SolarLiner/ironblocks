use std::fmt::{Debug, Display, Error, Formatter};
use std::ops::Deref;

use vst::util::AtomicFloat;

pub struct AtomicDecibel(AtomicFloat);

impl AtomicDecibel {
    pub fn new(decibel: f32) -> Self {
        Self(AtomicFloat::new(decibel))
    }
    pub fn from_linear(value: f32) -> Self {
        Self(AtomicFloat::new(linear2decibel(value)))
    }
    pub fn get_linear(&self) -> f32 {
        decibel2linear(self.0.get())
    }
    pub fn set_linear(&self, value: f32) {
        self.0.set(linear2decibel(value));
    }
}

impl Deref for AtomicDecibel {
    type Target = AtomicFloat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<AtomicFloat> for AtomicDecibel {
    fn as_ref(&self) -> &AtomicFloat {
        &self
    }
}

impl Display for AtomicDecibel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:.2} dB", self.get())
    }
}

impl Debug for AtomicDecibel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "AtomicDecibel({})", self.get())
    }
}

impl Default for AtomicDecibel {
    fn default() -> Self {
        Self(AtomicFloat::new(0.0))
    }
}

fn linear2decibel(v: f32) -> f32 {
    20.0 * v.log10()
}

fn decibel2linear(v: f32) -> f32 {
    10f32.powf(v / 20.0)
}

#[cfg(test)]
mod tests {
    use super::AtomicDecibel;

    #[test]
    fn it_works() {
        assert_eq!(0.0, AtomicDecibel::new(0.0).get());
    }

    #[test]
    fn from_linear() {
        assert_eq!(-6.0, AtomicDecibel::from_linear(0.5).get());
    }

    #[test]
    fn get_linear() {
        assert_eq!(0.5, AtomicDecibel::new(-6.0).get_linear());
    }

    #[test]
    fn set_linear() {
        let adb = AtomicDecibel::new(0.0);
        adb.set_linear(0.5);
        assert_eq!(-6.0, adb.get());
    }
}
