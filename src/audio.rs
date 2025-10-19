use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};

pub struct AudioRecorder {
    recording: Arc<Mutex<bool>>,
    output_path: Option<PathBuf>,
    writer: Option<Arc<Mutex<Option<WavWriter<std::io::BufWriter<std::fs::File>>>>>>,
    meter: Arc<AtomicU32>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            recording: Arc::new(Mutex::new(false)),
            output_path: None,
            writer: None,
            meter: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn start_recording(&mut self, output_path: PathBuf) -> Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No input device available")?;

        let config = device
            .default_input_config()
            .context("Failed to get default input config")?;

        log::info!("Input device: {}", device.name()?);
        log::info!("Default input config: {:?}", config);

        *self.recording.lock().unwrap() = true;
        self.output_path = Some(output_path.clone());

        let spec = WavSpec {
            channels: config.channels(),
            sample_rate: config.sample_rate().0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = Arc::new(Mutex::new(Some(
            WavWriter::create(&output_path, spec).context("Failed to create WAV file")?,
        )));

        self.writer = Some(Arc::clone(&writer));
        let writer_clone = Arc::clone(&writer);
        let recording_clone = Arc::clone(&self.recording);
        let meter_clone = Arc::clone(&self.meter);

        let err_fn = |err| log::error!("An error occurred on stream: {}", err);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| {
                    if *recording_clone.lock().unwrap() {
                        let mut accum = 0.0f32;
                        if let Some(ref mut writer) = *writer_clone.lock().unwrap() {
                            for &sample in data {
                                let amplitude = (sample * i16::MAX as f32) as i16;
                                writer.write_sample(amplitude).unwrap();
                                accum += sample.abs();
                            }
                        }
                        if !data.is_empty() {
                            let avg = (accum / data.len() as f32).min(1.0);
                            meter_clone.store((avg * 1000.0) as u32, Ordering::Relaxed);
                        }
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &_| {
                    if *recording_clone.lock().unwrap() {
                        let mut accum = 0.0f32;
                        if let Some(ref mut writer) = *writer_clone.lock().unwrap() {
                            for &sample in data {
                                writer.write_sample(sample).unwrap();
                                accum += (sample as f32 / i16::MAX as f32).abs();
                            }
                        }
                        if !data.is_empty() {
                            let avg = (accum / data.len() as f32).min(1.0);
                            meter_clone.store((avg * 1000.0) as u32, Ordering::Relaxed);
                        }
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config.into(),
                move |data: &[u16], _: &_| {
                    if *recording_clone.lock().unwrap() {
                        let mut accum = 0.0f32;
                        if let Some(ref mut writer) = *writer_clone.lock().unwrap() {
                            for &sample in data {
                                let sample = (sample as i32 - 32768) as i16;
                                writer.write_sample(sample).unwrap();
                                accum += (sample as f32 / i16::MAX as f32).abs();
                            }
                        }
                        if !data.is_empty() {
                            let avg = (accum / data.len() as f32).min(1.0);
                            meter_clone.store((avg * 1000.0) as u32, Ordering::Relaxed);
                        }
                    }
                },
                err_fn,
                None,
            )?,
            _ => return Err(anyhow::anyhow!("Unsupported sample format")),
        };

        stream.play()?;

        // Keep stream alive
        std::mem::forget(stream);

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<PathBuf> {
        *self.recording.lock().unwrap() = false;
        self.meter.store(0, Ordering::Relaxed);

        let path = self
            .output_path
            .take()
            .context("No recording in progress")?;

        // Give some time for the stream to finish writing
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Finalize and close the WAV file properly
        if let Some(writer_arc) = self.writer.take() {
            let mut writer_guard = writer_arc.lock().unwrap();
            if let Some(writer) = writer_guard.take() {
                log::info!("Finalizing WAV file...");
                writer.finalize().context("Failed to finalize WAV file")?;
                log::info!("WAV file finalized successfully");
            }
        }

        Ok(path)
    }

    pub fn is_recording(&self) -> bool {
        *self.recording.lock().unwrap()
    }

    pub fn meter(&self) -> Arc<AtomicU32> {
        Arc::clone(&self.meter)
    }
}
