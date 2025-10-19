use std::path::PathBuf;
use std::sync::{atomic::AtomicU32, Arc, Mutex};

use anyhow::Result;

use crate::audio::AudioRecorder;

/// Provides synchronized access to the audio recorder.
pub struct RecorderService {
    recorder: Mutex<AudioRecorder>,
    meter: Arc<AtomicU32>,
}

impl RecorderService {
    pub fn new(recorder: AudioRecorder) -> Self {
        let meter = recorder.meter();
        Self {
            recorder: Mutex::new(recorder),
            meter,
        }
    }

    pub fn start(&self, output_path: PathBuf) -> Result<()> {
        self.recorder
            .lock()
            .expect("recorder poisoned")
            .start_recording(output_path)
    }

    pub fn stop(&self) -> Result<PathBuf> {
        self.recorder
            .lock()
            .expect("recorder poisoned")
            .stop_recording()
    }

    pub fn is_recording(&self) -> bool {
        self.recorder
            .lock()
            .expect("recorder poisoned")
            .is_recording()
    }

    pub fn meter(&self) -> Arc<AtomicU32> {
        Arc::clone(&self.meter)
    }
}
