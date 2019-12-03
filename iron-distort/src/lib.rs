use std::fmt::{Debug, Error, Formatter};
use std::sync::Arc;

use vst::buffer::AudioBuffer;
use vst::plugin::{Info, Plugin, PluginParameters};
use vst::plugin_main;
use vst::util::AtomicFloat;

use atomic_db::AtomicDecibel;

type ProcessFn = dyn Fn(f32) -> f32;

const MAX_DRIVE: f32 = 100.0;
const MAX_DRIVE_DB: f32 = 40.0;

struct DistortionFunction {
    name: String,
    func: dyn Fn(Arc<DistortParameters>, f32) -> f32,
}

#[derive(Default, Debug)]
struct IronDistort {
    params: Arc<DistortParameters>,
}

struct DistortParameters {
    drive: AtomicDecibel,
    out_gain: AtomicDecibel,
    fnidx: AtomicFloat,
}

impl IronDistort {
    pub fn new(drive: f32) -> Self {
        Self {
            params: Arc::new(DistortParameters {
                drive: AtomicDecibel::from_linear(drive),
                ..Default::default()
            }),
        }
    }
}

impl Plugin for IronDistort {
    fn get_info(&self) -> Info {
        Info {
            name: "Iron Distort".to_owned(),
            vendor: "SolarLiner / Torcrafter".to_owned(),
            version: 1,
            inputs: 2,
            outputs: 2,
            parameters: 3,
            category: vst::plugin::Category::Effect,
            unique_id: -4432,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let drive = self.params.drive.get_linear().max(1e-10);
        let r#fn = match self.params.fnidx.get() {
            0.0..=0.33 => Box::new(move |p: f32| (p * drive).tanh() / drive.tanh().max(1e-10)) as Box<ProcessFn>,
            0.33..=0.67 => Box::new(move |p: f32| (p * drive).sin() / drive.min(1.0).sin().max(1e-10)) as Box<ProcessFn>,
            0.67..=1.0 => Box::new(move |p: f32| {
                let sign = if p > 0.0 { 1.0 } else { -1.0 };
                let val = ((p * drive).abs() + 1.0).ln() / (drive + 1.0).ln().max(1e-10);
                sign * val
            }) as Box<ProcessFn>,
            _ => Box::new(|p| p) as Box<ProcessFn>,
        };
        let (inputs, outputs) = buffer.split();
        if inputs.len() < 2 || outputs.len() < 2 {
            return;
        }
        let (l, r) = inputs.split_at(1);
        let stereo_in = l[0].iter().zip(r[0].iter());
        let (mut l, mut r) = outputs.split_at_mut(1);
        let stereo_out = l[0].iter_mut().zip(r[0].iter_mut());
        let out_gain = self.params.out_gain.get_linear();
        for ((lin, rin), (lout, rout)) in stereo_in.zip(stereo_out) {
            *lout = r#fn(*lin) * out_gain;
            *rout = r#fn(*rin) * out_gain;
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for DistortParameters {
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => format!("Drive"),
            1 => format!("Out Gain"),
            2 => format!("Dist. function"),
            _ => format!("Not Implemented"),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}", self.drive),
            1 => format!("{}", self.out_gain),
            2 => match self.fnidx.get() {
                0.0..=0.33 => format!("Hyp. Tengant"),
                0.33..=0.67 => format!("Sine"),
                0.67..=1.0 => format!("Log"),
                x => format!("Unknown range {:.2}", x)
            }
            _ => format!("Not Implemented"),
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.drive.get() / MAX_DRIVE_DB,
            1 => self.out_gain.get_linear(),
            2 => self.fnidx.get(),
            _ => unreachable!(),
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.drive.set(value * MAX_DRIVE_DB),
            1 => self.out_gain.set_linear(value),
            2 => self.fnidx.set(value),
            _ => (),
        }
    }

    fn can_be_automated(&self, index: i32) -> bool {
        match index {
            0 | 1 => true,
            _ => false
        }
    }
}

impl Default for DistortParameters {
    fn default() -> Self {
        Self {
            fnidx: AtomicFloat::new(0.0),
            drive: AtomicDecibel::new(0.0),
            out_gain: AtomicDecibel::new(0.0),
        }
    }
}

impl Debug for DistortParameters {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "DistortParameters(<Atomic Values>)")
    }
}

plugin_main!(IronDistort);
