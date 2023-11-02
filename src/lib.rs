use atomic_float::AtomicF32;
use egui::{Color32, Visuals};
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::sync::Arc;

pub struct Pan {
    params: Arc<PanParams>,
}

#[derive(Params)]
pub struct PanParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "pan"]
    pub pan: FloatParam,
}

impl Default for Pan {
    fn default() -> Self {
        Self {
            params: Arc::new(PanParams::default()),
        }
    }
}

impl Default for PanParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(300, 180),

            pan: FloatParam::new(
                "Pan",
                0.0, // Default pan value (centered)
                FloatRange::Linear {
                    min: -1.0, // Full left
                    max: 1.0,  // Full right
                },
            )
            .with_string_to_value(formatters::s2v_f32_panning()),
        }
    }
}

impl Plugin for Pan {
    const NAME: &'static str = "MinimalVST - Pan";
    const VENDOR: &'static str = "DanceMore";
    const URL: &'static str = "https://github.com/DanceMore/minimal_vst_pan";
    const EMAIL: &'static str = "dancemore@protonmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                let custom_theme = Visuals {
                    dark_mode: true,
                    //override_text_color: Some(Color32::WHITE),
                    override_text_color: Some(Color32::from_rgb(0, 166, 251)),
                    panel_fill: Color32::from_rgb(5, 25, 35),
                    // Set other color values and visual properties as needed.
                    // ...
                    ..Default::default() // Use default values for the fields you don't modify.
                };

                egui_ctx.set_visuals(custom_theme);

                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    // NOTE: See `plugins/diopser/src/editor.rs` for an example using the generic UI widget

                    // This is a fancy widget that can get all the information it needs to properly
                    // display and modify the parameter from the parametr itself
                    // It's not yet fully implemented, as the text is missing.

                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("Pan").underline());
                        ui.add(
                            widgets::ParamSlider::for_param(&params.pan, setter).with_width(200.0),
                        );
                    });
                });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for mut channel_samples in buffer.iter_samples() {
            let pan = self.params.pan.smoothed.next();

            // Calculate gains for left and right channels based on pan value
            let left_gain = (1.0 - pan) / 2.0;
            let right_gain = (1.0 + pan) / 2.0;

            // Apply the pan by adjusting the gain for each sample
            for (sample0, sample1) in channel_samples
                .iter_mut()
                .zip(channel_samples.iter_mut().skip(1))
            {
                *sample0 = *sample0 * left_gain; // Adjust gain for the left channel
                *sample1 = *sample1 * right_gain; // Adjust gain for the right channel
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Pan {
    const CLAP_ID: &'static str = "io.github.dancemore.minimal_vst_pan";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A minimal Pan plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for Pan {
    const VST3_CLASS_ID: [u8; 16] = *b"DancemoreMVSTPan";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(Pan);
nih_export_vst3!(Pan);
