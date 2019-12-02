use std::sync::Arc;

use vst::buffer::AudioBuffer;
use vst::plugin::{Info, Plugin, PluginParameters};
use vst::plugin_main;

use atomic_db::AtomicDecibel;

const MAX_DRIVE: f32 = 100.0;

#[derive(Default, Debug)]
struct IronDistort {
    params: Arc<DistortParameters>,
}

#[derive(Debug, Default)]
struct DistortParameters {
    drive: AtomicDecibel,
    out_gain: AtomicDecibel,
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
            parameters: 2,
            category: vst::plugin::Category::Effect,
            unique_id: -4432,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (inputs, outputs) = buffer.split();
        if inputs.len() < 2 || outputs.len() < 2 {
            return;
        }
        let (l, r) = inputs.split_at(1);
        let stereo_in = l[0].iter().zip(r[0].iter());
        let (mut l, mut r) = outputs.split_at_mut(1);
        let stereo_out = l[0].iter_mut().zip(r[0].iter_mut());
        let drive = self.params.drive.get_linear().max(1e-10);
        let out_gain = self.params.out_gain.get_linear();
        for ((lin, rin), (lout, rout)) in stereo_in.zip(stereo_out) {
            *lout = ((*lin * drive).tanh() / drive.min(1.0)) * out_gain;
            *rout = ((*rin * drive).tanh() / drive.min(1.0)) * out_gain;
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for DistortParameters {
    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "Drive".to_owned(),
            1 => "Out Gain".to_owned(),
            _ => "Not Implemented".to_owned(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("Drive: {}", self.drive),
            1 => format!("Out Gain: {}", self.out_gain),
            _ => format!("Not Implemented"),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        self.get_parameter_label(index)
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.drive.get_linear() / MAX_DRIVE,
            1 => self.out_gain.get_linear(),
            _ => unreachable!(),
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.drive.set_linear(value * MAX_DRIVE),
            1 => self.out_gain.set_linear(value),
            _ => (),
        }
    }

    fn can_be_automated(&self, _index: i32) -> bool {
        true
    }
}

fn map_param_db_range(val: f32) -> f32 {
    let out_min = -48.0;
    let out_max = 48.0;
    let out_range = out_max - out_min;
    let in_range = 1000.0;
    (val / in_range) * out_range + out_min
}

fn map_db_param_range(val: &AtomicDecibel) -> f32 {
    let out_range = 1000.0;
    let in_min = -48.0;
    let in_max = 48.0;
    let in_range = in_max - in_min;
    ((val.get() - in_min) / in_range) * out_range
}

plugin_main!(IronDistort);
