#![cfg(target_os = "macos")]

use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use core_graphics::event::{CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType, CGEventTapProxy};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};

const FN_KEY_MODIFIER: u64 = 0x800000; // NX_DEVICELFNFLAGSMASK - Fn key modifier on macOS

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FnKeyState {
    Pressed,
    Released,
}

pub struct FnKeyMonitor {
    receiver: Arc<Mutex<Receiver<FnKeyState>>>,
}

impl FnKeyMonitor {
    pub fn new() -> Self {
        log::info!("Initializing Fn key monitor...");
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            log::info!("Fn key monitoring thread started");
            Self::start_monitoring(tx);
        });

        Self {
            receiver: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn try_recv(&self) -> Option<FnKeyState> {
        self.receiver.lock().unwrap().try_recv().ok()
    }

    fn start_monitoring(sender: Sender<FnKeyState>) {
        let sender = Arc::new(Mutex::new(sender));
        let sender_clone = sender.clone();

        match CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::FlagsChanged],
            move |_proxy: CGEventTapProxy, _event_type: CGEventType, event: &CGEvent| -> Option<CGEvent> {
                // Get the modifier flags
                let flags = event.get_flags();
                let fn_pressed = (flags.bits() & FN_KEY_MODIFIER) != 0;

                // Track state to detect changes
                static LAST_FN_STATE: Mutex<bool> = Mutex::new(false);

                let mut last_state = LAST_FN_STATE.lock().unwrap();
                if fn_pressed != *last_state {
                    *last_state = fn_pressed;
                    let state = if fn_pressed {
                        FnKeyState::Pressed
                    } else {
                        FnKeyState::Released
                    };
                    log::info!("Fn key event detected: {:?}", state);
                    if let Ok(sender) = sender_clone.lock() {
                        let _ = sender.send(state);
                    }
                }

                None // Don't modify the event
            },
        ) {
            Ok(tap) => {
                log::info!("Fn key event tap created successfully");
                unsafe {
                    let loop_source = tap
                        .mach_port
                        .create_runloop_source(0)
                        .expect("Failed to create runloop source");

                    let run_loop = CFRunLoop::get_current();
                    run_loop.add_source(&loop_source, kCFRunLoopCommonModes);

                    tap.enable();
                    log::info!("Fn key event tap enabled, starting runloop");
                    CFRunLoop::run_current();
                }
            }
            Err(()) => {
                log::error!("Failed to create event tap for Fn key monitoring");
                log::error!("Note: You need to grant Accessibility permissions in System Settings > Privacy & Security > Accessibility");
            }
        }
    }
}
