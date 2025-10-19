mod theme;

use iced::theme::{Button, Theme};
use iced::time;
use iced::widget::{
    button, column, container, row, scrollable, svg, text, toggler,
};
use iced::{
    executor, window, Alignment, Application, Border, Color, Command, Element, Font, Length, Settings,
    Subscription,
};
use chrono::{DateTime, Utc, Local};
use theme::WillowDark;

// IBM Plex Mono font
const IBM_PLEX_MONO: &[u8] = include_bytes!("../../fonts/IBMPlexMono-Regular.otf");
const IBM_PLEX_MONO_MEDIUM: &[u8] = include_bytes!("../../fonts/IBMPlexMono-Medium.otf");
const IBM_PLEX_MONO_BOLD: &[u8] = include_bytes!("../../fonts/IBMPlexMono-Bold.otf");

// Heart icon
const HEART_SVG: &[u8] = include_bytes!("../../assets/heart.svg");

// Iced doesn't support emoji fonts well, so we won't use this
// Instead we'll use Unicode symbols that IBM Plex Mono supports

use crate::{
    database::Transcription, notch::NotchOverlay, services::AppServices, storage::AppSettings,
    workflow,
};
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use once_cell::sync::Lazy;
use std::sync::{mpsc, Mutex, Arc};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "macos")]
use crate::fn_key_monitor::{FnKeyMonitor, FnKeyState};

static HOTKEY_MANAGER: Lazy<Mutex<GlobalHotKeyManager>> = Lazy::new(|| {
    Mutex::new(GlobalHotKeyManager::new().expect("Failed to initialize hotkey manager"))
});

static HOTKEY_EVENTS: Lazy<Mutex<mpsc::Receiver<HotKeyState>>> = Lazy::new(|| {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            let _ = tx.send(event.state);
        }
    });
    Mutex::new(rx)
});

#[cfg(target_os = "macos")]
static FN_KEY_MONITOR: Lazy<Arc<FnKeyMonitor>> = Lazy::new(|| {
    Arc::new(FnKeyMonitor::new())
});

pub fn run(services: AppServices) -> iced::Result {
    let mut settings = Settings::with_flags(services);

    // Load IBM Plex Mono as the default font
    settings.default_font = Font::with_name("IBM Plex Mono");
    settings.fonts = vec![
        IBM_PLEX_MONO.into(),
        IBM_PLEX_MONO_MEDIUM.into(),
        IBM_PLEX_MONO_BOLD.into(),
    ];

    settings.window = window::Settings {
        decorations: true,
        size: iced::Size::new(900.0, 700.0),
        ..window::Settings::default()
    };
    App::run(settings)
}

// Removed Tab enum - no longer using tabs in Willow design

#[derive(Clone, Debug)]
pub enum Message {
    Initialize,
    SettingsLoaded(Result<AppSettings, String>),
    HistoryLoaded(Result<Vec<Transcription>, String>),
    RecordPressed,
    RecordingStarted(Result<(), String>),
    StopPressed,
    RecordingStopped(Result<String, String>),
    ToggleAutoPaste(bool),
    ToggleRecognizePressEnter(bool),
    SettingsSaved(Result<(), String>),
    HistoryDelete(i64),
    HistoryCopied(String),
    PollHotkey,
}

pub struct App {
    services: AppServices,
    settings: Option<AppSettings>,
    settings_draft: Option<AppSettings>,
    settings_saving: bool,
    history: Vec<Transcription>,
    is_recording: bool,
    is_processing: bool,
    last_transcription: Option<String>,
    error: Option<String>,
    notch_overlay: NotchOverlay,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = AppServices;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        Lazy::force(&HOTKEY_MANAGER);
        Lazy::force(&HOTKEY_EVENTS);

        let overlay = NotchOverlay::new(flags.recorder.meter());
        (
            Self {
                services: flags,
                settings: None,
                settings_draft: None,
                settings_saving: false,
                history: Vec::new(),
                is_recording: false,
                is_processing: false,
                last_transcription: None,
                error: None,
                notch_overlay: overlay,
            },
            Command::perform(async {}, |_| Message::Initialize),
        )
    }

    fn title(&self) -> String {
        "Convey".into()
    }

    fn theme(&self) -> Self::Theme {
        // Custom theme with Solarized-inspired colors
        Theme::custom(
            String::from("Willow"),
            iced::theme::Palette {
                background: WillowDark::BACKGROUND,
                text: WillowDark::TEXT_PRIMARY,
                primary: WillowDark::ACCENT,
                success: WillowDark::SUCCESS,
                danger: WillowDark::ERROR,
            },
        )
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_millis(50)).map(|_| Message::PollHotkey)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Initialize => {
                let services = self.services.clone();
                let services_history = self.services.clone();
                Command::batch(vec![
                    Command::perform(
                        async move { services.settings.load().map_err(|e| e.to_string()) },
                        Message::SettingsLoaded,
                    ),
                    Command::perform(
                        async move {
                            services_history
                                .history
                                .recent(50)
                                .map_err(|e| e.to_string())
                        },
                        Message::HistoryLoaded,
                    ),
                ])
            }
            Message::SettingsLoaded(result) => {
                match result {
                    Ok(settings) => {
                        self.settings_draft = Some(settings.clone());
                        self.settings = Some(settings);
                    }
                    Err(err) => self.error = Some(err),
                }
                Command::none()
            }
            Message::HistoryLoaded(result) => {
                match result {
                    Ok(list) => self.history = list,
                    Err(err) => self.error = Some(err),
                }
                Command::none()
            }
            Message::RecordPressed => {
                return self.start_recording_command();
            }
            Message::RecordingStarted(result) => {
                self.is_processing = false;
                match result {
                    Ok(_) => {
                        self.is_recording = true;
                        self.notch_overlay.show_recording();
                    }
                    Err(err) => {
                        self.error = Some(err);
                        self.notch_overlay.hide();
                    }
                }
                Command::none()
            }
            Message::StopPressed => {
                return self.stop_recording_command();
            }
            Message::RecordingStopped(result) => {
                self.is_processing = false;
                self.notch_overlay.hide();
                match result {
                    Ok(text) => {
                        self.last_transcription = Some(text);
                        self.is_recording = false;
                        let services = self.services.clone();
                        return Command::perform(
                            async move { services.history.recent(50).map_err(|e| e.to_string()) },
                            Message::HistoryLoaded,
                        );
                    }
                    Err(err) => {
                        self.error = Some(err);
                    }
                }
                self.is_recording = false;
                Command::none()
            }
            Message::ToggleAutoPaste(value) => {
                if let Some(settings) = &mut self.settings_draft {
                    settings.auto_paste = value;
                    // Auto-save
                    return self.save_settings_command();
                }
                Command::none()
            }
            Message::ToggleRecognizePressEnter(value) => {
                if let Some(settings) = &mut self.settings_draft {
                    settings.recognize_press_enter = value;
                    // Auto-save
                    return self.save_settings_command();
                }
                Command::none()
            }
            Message::SettingsSaved(result) => {
                self.settings_saving = false;
                match result {
                    Ok(_) => {
                        if let Some(draft) = &self.settings_draft {
                            self.settings = Some(draft.clone());
                        }
                    }
                    Err(err) => self.error = Some(err),
                }
                Command::none()
            }
            Message::HistoryDelete(id) => {
                let services = self.services.clone();
                Command::perform(
                    async move {
                        services.history.delete(id).map_err(|e| e.to_string())?;
                        services.history.recent(50).map_err(|e| e.to_string())
                    },
                    Message::HistoryLoaded,
                )
            }
            Message::HistoryCopied(text) => {
                if let Err(err) = self.services.clipboard.copy_text(&text) {
                    self.error = Some(err.to_string());
                }
                Command::none()
            }
            Message::PollHotkey => {
                // Check for Fn key events first (macOS only)
                #[cfg(target_os = "macos")]
                {
                    if let Some(settings) = &self.settings {
                        if settings.hotkey.to_lowercase() == "fn" || settings.hotkey.to_lowercase() == "globe" {
                            Lazy::force(&FN_KEY_MONITOR);
                            if let Some(fn_state) = FN_KEY_MONITOR.try_recv() {
                                log::info!("Received Fn key state in UI: {:?}", fn_state);
                                match fn_state {
                                    FnKeyState::Pressed => {
                                        log::info!("Fn pressed - starting recording");
                                        if !self.is_recording && !self.is_processing {
                                            return self.start_recording_command();
                                        }
                                    }
                                    FnKeyState::Released => {
                                        log::info!("Fn released - stopping recording");
                                        if self.is_recording && !self.is_processing {
                                            return self.stop_recording_command();
                                        }
                                    }
                                }
                            }
                            return Command::none();
                        }
                    }
                }

                // Fall back to global-hotkey for other keys
                let events: Vec<HotKeyState> = {
                    let guard = HOTKEY_EVENTS.lock().unwrap();
                    let mut collected = Vec::new();
                    while let Ok(state) = guard.try_recv() {
                        collected.push(state);
                    }
                    collected
                };

                // Process events - use the last event if multiple occurred
                if let Some(last_state) = events.last() {
                    match last_state {
                        HotKeyState::Pressed => {
                            // Start recording when key is pressed
                            if !self.is_recording && !self.is_processing {
                                return self.start_recording_command();
                            }
                        }
                        HotKeyState::Released => {
                            // Stop recording when key is released
                            if self.is_recording && !self.is_processing {
                                return self.stop_recording_command();
                            }
                        }
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // Settings section - positioned top right
        let settings_section = self.inline_settings_view();

        // Top bar with settings on right
        let top_bar = row![
            row![].width(Length::Fill), // Spacer to push settings to the right
            settings_section,
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill);

        // Hero recording card
        let hero_card = self.record_view();

        // Recent transcriptions list
        let recent_list = self.recent_transcriptions_view();

        // Footer with attribution
        let heart_icon = svg(svg::Handle::from_memory(HEART_SVG))
            .width(12)
            .height(12);

        let footer = container(
            row![
                text("Made with ").size(12).style(WillowDark::TEXT_MUTED),
                heart_icon,
                text(" by Naren Laxmidas").size(12).style(WillowDark::TEXT_MUTED),
            ]
            .spacing(4)
            .align_items(Alignment::Center)
        )
        .center_x()
        .width(Length::Fill)
        .padding([8, 0]);

        // Main content area
        // 8px-based grid system:
        // - Layout margin: 24px (3 units)
        // - Section spacing: 32px (4 units)
        // - Card spacing: 16px (2 units)
        let main_content = column![top_bar, hero_card, recent_list, footer]
            .spacing(32)
            .width(Length::Fill);

        // Consistent layout margin: 24px
        container(main_content)
            .padding(24)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(main_container_style())
            .into()
    }
}

impl App {
    fn record_view(&self) -> Element<'_, Message> {
        let button_text = if self.is_recording {
            "Release to transcribe"
        } else if self.is_processing {
            "Transcribing..."
        } else {
            return container(
                button(text("Press Globe/Fn to transcribe").size(16))
                    .padding([16, 32])
                    .style(animated_primary_style())
                    .on_press(Message::RecordPressed)
            )
            .center_x()
            .width(Length::Fill)
            .into();
        };

        let mut record_button = button(text(button_text).size(16))
            .padding([16, 32])
            .style(animated_primary_style());

        if self.is_processing {
            record_button = record_button.style(Button::Secondary);
        } else if self.is_recording {
            record_button = record_button.on_press(Message::StopPressed);
        } else {
            record_button = record_button.on_press(Message::RecordPressed);
        }

        let mut main_column = column![
            container(record_button)
                .center_x()
                .width(Length::Fill),
        ]
        .spacing(20)
        .align_items(Alignment::Center)
        .width(Length::Fill);

        if let Some(err) = &self.error {
            main_column = main_column.push(
                container(
                    row![
                        text("Error:").size(16).style(WillowDark::ERROR),
                        text(err.clone()).size(16).style(WillowDark::ERROR)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                )
                .padding(16)
                .style(modern_card_style())
                .width(Length::Fill)
                .max_width(600),
            );
        }

        container(main_column)
            .padding([40, 0])
            .width(Length::Fill)
            .into()
    }

    fn inline_settings_view(&self) -> Element<'_, Message> {
        if let Some(settings) = &self.settings {
            let draft = self.settings_draft.as_ref().unwrap_or(settings);

            // Auto paste toggle
            let auto_paste_toggle = toggler(
                Some("Auto paste".to_string()),
                draft.auto_paste,
                Message::ToggleAutoPaste,
            )
            .text_size(14)
            .spacing(8)
            .width(Length::Shrink);

            // Show "Recognize 'and press enter'" toggle only when auto_paste is enabled
            let toggles_row = if draft.auto_paste {
                row![
                    auto_paste_toggle,
                    toggler(
                        Some("Recognize 'and press enter'".to_string()),
                        draft.recognize_press_enter,
                        Message::ToggleRecognizePressEnter,
                    )
                    .text_size(14)
                    .spacing(8)
                    .width(Length::Shrink),
                ]
                .spacing(24)
                .align_items(Alignment::Center)
            } else {
                row![auto_paste_toggle]
                .spacing(24)
                .align_items(Alignment::Center)
            };

            // Settings without card styling - aligns with layout margin
            toggles_row.into()
        } else {
            container(text(""))
                .width(Length::Fill)
                .into()
        }
    }

    fn recent_transcriptions_view(&self) -> Element<'_, Message> {
        if self.history.is_empty() {
            return container(
                text("No recent transcriptions")
                    .size(14)
                    .style(WillowDark::TEXT_MUTED),
            )
            .padding(20)
            .center_x()
            .width(Length::Fill)
            .into();
        }

        let header = text("Recent Transcriptions")
            .size(18)
            .style(WillowDark::TEXT_PRIMARY);

        let list = self
            .history
            .iter()
            .take(10)
            .map(|item| {
                let transcription_text = item.processed_text.as_ref().unwrap_or(&item.text).clone();
                let formatted_time = format_timestamp(&item.created_at);

                let copy_btn = button(text("Copy").size(13))
                    .padding([6, 12])
                    .style(subtle_button_style())
                    .on_press(Message::HistoryCopied(transcription_text.clone()));

                let delete_btn = button(text("Delete").size(13))
                    .padding([6, 12])
                    .style(subtle_button_style())
                    .on_press(Message::HistoryDelete(item.id));

                container(
                    column![
                        text(formatted_time)
                            .size(12)
                            .style(WillowDark::TEXT_MUTED),
                        text(&transcription_text)
                            .size(15)
                            .style(WillowDark::TEXT_SECONDARY),
                        row![copy_btn, delete_btn]
                            .spacing(10)
                            .align_items(Alignment::Center),
                    ]
                    .spacing(8),
                )
                .padding(20)
                .style(modern_card_style())
                .width(Length::Fill)
                .into()
            })
            .collect::<Vec<Element<_>>>();

        // Add right padding to prevent scrollbar from overlapping cards
        column![
            header,
            container(
                scrollable(
                    container(column(list).spacing(16))
                        .padding([0, 12, 0, 0])  // 12px right padding for scrollbar space
                        .width(Length::Fill)
                )
                .height(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
        ]
        .spacing(16)
        .width(Length::Fill)
        .into()
    }

    fn sanitize_settings(_settings: &mut AppSettings) {
        // No sanitization needed anymore
    }

    fn save_settings_command(&mut self) -> Command<Message> {
        if self.settings_saving {
            return Command::none();
        }
        if let Some(draft) = self.settings_draft.as_mut() {
            Self::sanitize_settings(draft);
            self.settings_saving = true;
            self.error = None;
            let services = self.services.clone();
            let draft_clone = draft.clone();
            Command::perform(
                async move {
                    services
                        .settings
                        .save(&draft_clone)
                        .map_err(|e| e.to_string())
                },
                Message::SettingsSaved,
            )
        } else {
            Command::none()
        }
    }

    fn start_recording_command(&mut self) -> Command<Message> {
        if self.is_processing || self.is_recording {
            return Command::none();
        }
        if self.services.recorder.is_recording() {
            self.is_recording = true;
            return Command::none();
        }
        self.error = None;
        self.notch_overlay.show_recording();
        let services = self.services.clone();
        Command::perform(
            workflow::start_recording(services),
            Message::RecordingStarted,
        )
    }

    fn stop_recording_command(&mut self) -> Command<Message> {
        if !self.is_recording || self.is_processing {
            return Command::none();
        }
        if !self.services.recorder.is_recording() {
            self.is_recording = false;
            return Command::none();
        }
        self.is_processing = true;
        self.notch_overlay.show_processing();
        let services = self.services.clone();
        Command::perform(
            async move { workflow::stop_recording_and_transcribe(services).await },
            Message::RecordingStopped,
        )
    }

    fn handle_hotkey_trigger(&mut self) -> Command<Message> {
        if self.is_processing {
            Command::none()
        } else if self.is_recording {
            self.stop_recording_command()
        } else {
            self.start_recording_command()
        }
    }

}

// Removed tab_button - no longer using tabs

// Helper function to format timestamp in a user-friendly way
fn format_timestamp(timestamp_str: &str) -> String {
    // Parse the RFC3339 timestamp from the database
    if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp_str) {
        let now = Utc::now();
        let duration = now.signed_duration_since(dt.with_timezone(&Utc));

        // Format based on how recent it is
        if duration.num_seconds() < 60 {
            "Just now".to_string()
        } else if duration.num_minutes() < 60 {
            let mins = duration.num_minutes();
            format!("{} min{} ago", mins, if mins == 1 { "" } else { "s" })
        } else if duration.num_hours() < 24 {
            let hours = duration.num_hours();
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else if duration.num_days() < 7 {
            let days = duration.num_days();
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        } else {
            // For older items, show the date
            let local_dt = dt.with_timezone(&Local);
            local_dt.format("%b %d, %I:%M %p").to_string()
        }
    } else {
        // Fallback if parsing fails
        timestamp_str.to_string()
    }
}

fn modern_card_style() -> impl Fn(&iced::Theme) -> iced::widget::container::Appearance {
    |_theme| {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(WillowDark::SURFACE)),
            border: Border {
                radius: 12.0.into(),
                width: 1.0,
                color: WillowDark::SURFACE_BORDER,
            },
            text_color: Some(WillowDark::TEXT_PRIMARY),
            ..Default::default()
        }
    }
}

fn main_container_style() -> impl Fn(&iced::Theme) -> iced::widget::container::Appearance {
    |_theme| {
        use crate::ui::theme::WillowDark;
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(WillowDark::BACKGROUND)),
            border: Border::default(),
            text_color: Some(WillowDark::TEXT_PRIMARY),
            ..Default::default()
        }
    }
}

// Custom button style for subtle interactions
fn subtle_button_style() -> iced::theme::Button {
    iced::theme::Button::Custom(Box::new(SubtleButtonStyle))
}

struct SubtleButtonStyle;

impl iced::widget::button::StyleSheet for SubtleButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(WillowDark::SURFACE)),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: WillowDark::BORDER,
            },
            text_color: WillowDark::TEXT_SECONDARY,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(WillowDark::SURFACE_HOVER)),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: WillowDark::ACCENT,
            },
            text_color: WillowDark::ACCENT,
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(
                0xd8 as f32 / 255.0,
                0xd2 as f32 / 255.0,
                0xbf as f32 / 255.0,
            ))),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: WillowDark::ACCENT,
            },
            text_color: WillowDark::ACCENT,
            ..Default::default()
        }
    }
}

// Custom primary button with animation feel
fn animated_primary_style() -> iced::theme::Button {
    iced::theme::Button::Custom(Box::new(AnimatedPrimaryStyle))
}

struct AnimatedPrimaryStyle;

impl iced::widget::button::StyleSheet for AnimatedPrimaryStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(WillowDark::ACCENT)),
            border: Border {
                radius: 12.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: WillowDark::BACKGROUND,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(
                0x34 as f32 / 255.0,
                0x9f as f32 / 255.0,
                0xe6 as f32 / 255.0,
            ))),
            border: Border {
                radius: 12.0.into(),
                width: 2.0,
                color: WillowDark::ACCENT,
            },
            text_color: WillowDark::BACKGROUND,
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(
                0x1e as f32 / 255.0,
                0x77 as f32 / 255.0,
                0xbe as f32 / 255.0,
            ))),
            border: Border {
                radius: 12.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: WillowDark::BACKGROUND,
            ..Default::default()
        }
    }
}



