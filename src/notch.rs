#![allow(unexpected_cfgs)]

use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use dispatch::Queue;
use objc::rc::StrongPtr;
use objc::runtime::{Object, NO, YES};
use objc::{class, msg_send, sel, sel_impl};

const NOTCH_WIDTH: f64 = 200.0; // Approximate width of the physical notch
const PANEL_WIDTH: f64 = 320.0; // Total width - minimal horizontal expansion
const ICON_SIZE: f64 = 20.0;
const ICON_MARGIN: f64 = 10.0;
const BAR_COUNT: usize = 18;
const BAR_WIDTH: f64 = 3.0;
const BAR_SPACING: f64 = 3.0;
const PANEL_VERTICAL_MARGIN: f64 = 10.0;
const NOTCH_THRESHOLD: f64 = 30.0;
const OVERLAY_WINDOW_LEVEL: i32 = 2147483631; // CGShieldingWindowLevel - highest possible level
const FLOATING_WINDOW_LEVEL: i32 = 5; // NSFloatingWindowLevel - above most windows
const COLLECTION_BEHAVIOR: u64 = (1 << 0) | (1 << 4) | (1 << 6); // CanJoinAllSpaces | Stationary | IgnoresCycle

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct NSPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct NSSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct NSEdgeInsets {
    top: f64,
    left: f64,
    bottom: f64,
    right: f64,
}

fn nsrect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}

fn nscolor(r: f64, g: f64, b: f64, a: f64) -> *mut Object {
    unsafe { msg_send![class!(NSColor), colorWithCalibratedRed:r green:g blue:b alpha:a] }
}

pub struct NotchOverlay {
    panel: Option<StrongPtr>,
    panel_ptr: Option<*mut Object>,
    panel_height: f64,
    icon_view: Option<StrongPtr>,
    bars: Vec<StrongPtr>,
    bar_ptrs: Vec<usize>,
    running: Arc<AtomicBool>,
    processing: Arc<AtomicBool>,
    meter: Arc<AtomicU32>,
    update_handle: Option<JoinHandle<()>>,
}

impl NotchOverlay {
    pub fn new(meter: Arc<AtomicU32>) -> Self {
        Self {
            panel: None,
            panel_ptr: None,
            panel_height: 44.0, // Default, will be updated when panel is created
            icon_view: None,
            bars: Vec::new(),
            bar_ptrs: Vec::new(),
            running: Arc::new(AtomicBool::new(false)),
            processing: Arc::new(AtomicBool::new(false)),
            meter,
            update_handle: None,
        }
    }

    pub fn show_recording(&mut self) {
        self.processing.store(false, Ordering::Relaxed);
        self.show();
    }

    pub fn show_processing(&mut self) {
        self.processing.store(true, Ordering::Relaxed);
        self.show();
    }

    pub fn hide(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.meter.store(0, Ordering::Relaxed);
        if let Some(handle) = self.update_handle.take() {
            let _ = handle.join();
        }
        if let Some(panel_ptr) = self.panel_ptr {
            unsafe {
                // Simple fade out
                let _: () = msg_send![panel_ptr, setAlphaValue:0.0f64];
                let _: () = msg_send![panel_ptr, orderOut: nil];
            }
        }
    }

    fn show(&mut self) {
        unsafe {
            self.ensure_panel();
            if let Some(panel_ptr) = self.panel_ptr {
                log::info!("Showing notch overlay panel");
                // Show immediately - no animation delays
                let _: () = msg_send![panel_ptr, setAlphaValue:1.0f64];
                let _: () = msg_send![panel_ptr, orderFrontRegardless];
                let _: () = msg_send![panel_ptr, makeKeyAndOrderFront:nil];

                // Verify it's visible
                let is_visible: bool = msg_send![panel_ptr, isVisible];
                let level: i32 = msg_send![panel_ptr, level];
                log::info!("Panel visible: {}, level: {}", is_visible, level);
            }
        }
        self.start_updates();
    }

    fn ensure_panel(&mut self) {
        if self.panel.is_some() {
            return;
        }

        unsafe {
            // Calculate the actual notch height from screen geometry
            let screen: *mut Object = msg_send![class!(NSScreen), mainScreen];
            let frame: NSRect = msg_send![screen, frame];
            let visible: NSRect = msg_send![screen, visibleFrame];

            // Use safeAreaInsets to get the actual notch dimensions (macOS 12+)
            let safe_insets: NSEdgeInsets = msg_send![screen, safeAreaInsets];
            let notch_height = safe_insets.top; // top inset is the notch height

            log::info!("Safe area insets - top: {}, left: {}, bottom: {}, right: {}",
                safe_insets.top, safe_insets.left, safe_insets.bottom, safe_insets.right);

            // Match the notch height exactly - extend a bit more to ensure we cover the entire notch
            let panel_height = if notch_height > 0.0 {
                notch_height + 2.0 // Add 2px to ensure full coverage
            } else {
                44.0 // Fallback for non-notch displays
            };

            // Position the panel just below the notch in the visible area
            // This is simpler and more reliable than trying to draw IN the notch
            let panel_x = (frame.size.width - PANEL_WIDTH) / 2.0;
            let panel_y = visible.origin.y + visible.size.height - panel_height - 8.0; // 8px below menubar

            let panel_rect = nsrect(panel_x, panel_y, PANEL_WIDTH, panel_height);

            // Use NSPanel with high window level
            let panel: *mut Object = msg_send![class!(NSPanel), alloc];
            let panel: *mut Object = msg_send![panel,
                initWithContentRect:panel_rect
                styleMask:1u64 // NSWindowStyleMaskBorderless
                backing:2u64
                defer:NO
            ];

            log::info!("Creating panel below notch at x={}, y={} (panel_height={})", panel_x, panel_y, panel_height);

            let _: () = msg_send![panel, setTitleVisibility:1u64];
            let _: () = msg_send![panel, setTitlebarAppearsTransparent:YES];
            let _: () = msg_send![panel, setOpaque:NO]; // Transparent window
            let _: () = msg_send![panel, setHasShadow:NO]; // No shadow for seamless notch effect
            let _: () = msg_send![panel, setLevel:OVERLAY_WINDOW_LEVEL];
            let _: () = msg_send![panel, setIgnoresMouseEvents:YES];
            let _: () = msg_send![panel, setCollectionBehavior:COLLECTION_BEHAVIOR];
            let _: () = msg_send![panel, setFloatingPanel:YES]; // Allow panel to float above everything
            let _: () = msg_send![panel, setWorksWhenModal:YES]; // Keep working in modal contexts
            let _: () = msg_send![panel, setCanHide:NO]; // Prevent hiding
            let _: () = msg_send![panel, setHidesOnDeactivate:NO]; // Stay visible when app loses focus
            // Set transparent background for the full screen window
            let clear = nscolor(0.0, 0.0, 0.0, 0.0);
            let _: () = msg_send![panel, setBackgroundColor:clear];

            // Create a content view that fills the small panel
            let content_view: *mut Object = msg_send![class!(NSView), alloc];
            let content_view: *mut Object = msg_send![content_view, initWithFrame:panel_rect];
            let _: () = msg_send![content_view, setWantsLayer:YES];

            let _: () = msg_send![panel, setContentView:content_view];

            // Create the notch bar that fills the entire panel (since the panel is already sized correctly)
            let notch_rect = nsrect(0.0, 0.0, PANEL_WIDTH, panel_height);

            log::info!("Creating notch bar filling panel: width={}, height={}", PANEL_WIDTH, panel_height);

            let notch_bar: *mut Object = msg_send![class!(NSView), alloc];
            let notch_bar: *mut Object = msg_send![notch_bar, initWithFrame:notch_rect];
            let _: () = msg_send![notch_bar, setWantsLayer:YES];

            // Ensure notch bar's layer can draw outside bounds
            let notch_layer: *mut Object = msg_send![notch_bar, layer];
            let _: () = msg_send![notch_layer, setMasksToBounds:NO];

            let _: () = msg_send![content_view, addSubview:notch_bar];

            // Transparent background - no black bar, just the waveform
            let transparent = nscolor(0.0, 0.0, 0.0, 0.0);
            let layer: *mut Object = msg_send![notch_bar, layer];
            let cg_color: *mut Object = msg_send![transparent, CGColor];
            let _: () = msg_send![layer, setBackgroundColor: cg_color];

            log::info!("Notch bar created with width={}, height={}", PANEL_WIDTH, panel_height);

            // Create waveform bars centered in the panel
            let mut bars = Vec::with_capacity(BAR_COUNT);
            let mut bars_raw = Vec::with_capacity(BAR_COUNT);

            // Calculate total width of all bars
            let total_bars_width = (BAR_COUNT as f64 * BAR_WIDTH) + ((BAR_COUNT - 1) as f64 * BAR_SPACING);
            let bars_start_x = (PANEL_WIDTH - total_bars_width) / 2.0;

            let _max_bar_height = panel_height - 16.0; // Leave 8px padding top and bottom

            for i in 0..BAR_COUNT {
                let x = bars_start_x + i as f64 * (BAR_WIDTH + BAR_SPACING);
                let initial_height = 4.0;
                // Center bars vertically
                let bar_y = (panel_height - initial_height) / 2.0;
                let bar_frame = nsrect(x, bar_y, BAR_WIDTH, initial_height);
                let bar: *mut Object = msg_send![class!(NSView), alloc];
                let bar: *mut Object = msg_send![bar, initWithFrame:bar_frame];
                let _: () = msg_send![bar, setWantsLayer:YES];
                let layer: *mut Object = msg_send![bar, layer];

                // White color for better visibility on black background
                let bar_color = nscolor(1.0, 1.0, 1.0, 0.95);
                let cg_bar: *mut Object = msg_send![bar_color, CGColor];
                let _: () = msg_send![layer, setBackgroundColor: cg_bar];
                let _: () = msg_send![layer, setCornerRadius:1.75f64];
                let _: () = msg_send![notch_bar, addSubview:bar];
                bars_raw.push(bar as usize);
                bars.push(StrongPtr::new(bar));
            }

            self.icon_view = None; // No icon in simplified version

            self.bars = bars;
            self.bar_ptrs = bars_raw;
            self.panel = Some(StrongPtr::new(panel));
            self.panel_ptr = Some(panel);
            self.panel_height = panel_height;
        }
    }

    fn position_panel(&self, panel: *mut Object) {
        unsafe {
            let screen: *mut Object = msg_send![class!(NSScreen), mainScreen];
            if screen.is_null() {
                return;
            }
            let frame: NSRect = msg_send![screen, frame];
            let visible: NSRect = msg_send![screen, visibleFrame];
            let top_inset = (frame.size.height - visible.size.height - visible.origin.y).max(0.0);
            let has_notch = top_inset > NOTCH_THRESHOLD;

            log::info!("Screen frame: {:?}", frame);
            log::info!("Visible frame: {:?}", visible);
            log::info!("Top inset: {}, Has notch: {}", top_inset, has_notch);

            let (mut x, y) = if has_notch {
                // Position centered horizontally
                let center_x = frame.origin.x + (frame.size.width - PANEL_WIDTH) / 2.0;
                // Position at the ABSOLUTE TOP of the screen (into the notch)
                // The window will extend from top of screen downward
                let notch_y = visible.origin.y + visible.size.height;

                log::info!("Notch positioning (at top/in notch): x={}, y={}, panel_height={}, screen_top={}",
                    center_x, notch_y, self.panel_height, frame.origin.y + frame.size.height);
                (center_x, notch_y)
            } else {
                let horizontal_margin = 16.0;
                let vertical_margin = 12.0;
                (
                    visible.origin.x + visible.size.width - PANEL_WIDTH - horizontal_margin,
                    visible.origin.y + visible.size.height - self.panel_height - vertical_margin,
                )
            };
            let min_x = visible.origin.x + 12.0;
            if x < min_x {
                x = min_x;
            }
            let origin = NSPoint { x, y };
            log::info!("Final position: x={}, y={}", origin.x, origin.y);
            let _: () = msg_send![panel, setFrameOrigin:origin];
            if !has_notch {
                let _: () = msg_send![panel, setAlphaValue:0.96f64];
            }
        }
    }

    fn start_updates(&mut self) {
        if self.running.swap(true, Ordering::Relaxed) {
            return;
        }

        let meter = Arc::clone(&self.meter);
        let running = Arc::clone(&self.running);
        let processing = Arc::clone(&self.processing);
        let bars_ptrs = self.bar_ptrs.clone();
        let panel_height = self.panel_height;

        self.update_handle = Some(thread::spawn(move || {
            let queue = Queue::main();
            let mut phase = 0.0f64;
            while running.load(Ordering::Relaxed) {
                phase += 0.2;
                let amplitude = (meter.load(Ordering::Relaxed) as f64 / 1000.0).min(1.0);
                let is_processing = processing.load(Ordering::Relaxed);
                let max_height = panel_height - 16.0; // 8px padding top and bottom
                let heights: Vec<f64> = (0..BAR_COUNT)
                    .map(|i| {
                        let center = BAR_COUNT as f64 / 2.0;
                        let dist_from_center = ((i as f64 - center).abs() / center).min(1.0);

                        if is_processing {
                            // Processing: smooth wave propagation
                            let base = 4.0;
                            let sweep = (phase + i as f64 * 0.4).sin().abs();
                            (base + sweep * (max_height - base) * 0.7).min(max_height)
                        } else {
                            // Recording: symmetric waveform with high amplitude response
                            let base = 3.0;
                            let wave = ((phase + i as f64 * 0.5).sin() + 1.0) * 0.5;
                            let center_boost = 1.0 - (dist_from_center * 0.4);
                            let responsive_height = amplitude * max_height * center_boost * 0.85;
                            let idle_motion = wave * (max_height * 0.35);

                            if amplitude > 0.08 {
                                (base + responsive_height * (0.4 + wave * 0.6)).min(max_height)
                            } else {
                                (base + idle_motion).min(max_height)
                            }
                        }
                    })
                    .collect();

                let bars_clone = bars_ptrs.clone();
                queue.exec_async(move || unsafe {
                    for (bar_ptr, height) in bars_clone.iter().zip(heights.iter()) {
                        let bar_view = *bar_ptr as *mut Object;
                        let mut frame: NSRect = msg_send![bar_view, frame];
                        frame.size.height = *height;
                        // Center bar vertically
                        frame.origin.y = (panel_height - *height) / 2.0;
                        let _: () = msg_send![bar_view, setFrame:frame];
                    }
                });

                thread::sleep(Duration::from_millis(16));
            }
        }));
    }
}

impl Drop for NotchOverlay {
    fn drop(&mut self) {
        self.hide();
    }
}

#[allow(non_upper_case_globals)]
const nil: *mut Object = std::ptr::null_mut();
