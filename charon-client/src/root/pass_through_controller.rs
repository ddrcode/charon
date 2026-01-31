use std::time::{Duration, Instant};

use charond::domain::{qmk::QMKEvent, CharonEvent};

use crate::domain::AppEvent;

/// Controls which view is displayed during pass-through mode.
///
/// Handles transitions between default view (Charonsay), layer preview (Keymap),
/// and potentially other views like live stats in the future.
pub struct PassThroughController {
    /// Currently active view in pass-through mode
    active_view: PassThroughView,
    /// Default view to return to (user preference)
    default_view: PassThroughView,
    /// When a layer change was detected (for hesitation timing)
    layer_change_at: Option<Instant>,
    /// The layer we're waiting to preview
    pending_layer: Option<u8>,
    /// How long to wait before showing layer preview
    hesitation_duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassThroughView {
    Charonsay,
    Keymap,
    // Future: LiveStats, etc.
}

impl PassThroughView {
    pub fn app_id(&self) -> &'static str {
        match self {
            PassThroughView::Charonsay => "charonsay",
            PassThroughView::Keymap => "keymap",
        }
    }
}

impl Default for PassThroughController {
    fn default() -> Self {
        Self {
            active_view: PassThroughView::Charonsay,
            default_view: PassThroughView::Charonsay,
            layer_change_at: None,
            pending_layer: None,
            hesitation_duration: Duration::from_millis(500),
        }
    }
}

impl PassThroughController {
    pub fn new(hesitation_ms: u64) -> Self {
        Self {
            hesitation_duration: Duration::from_millis(hesitation_ms),
            ..Default::default()
        }
    }

    /// Returns the app ID of the currently active pass-through view
    pub fn active_app(&self) -> &'static str {
        self.active_view.app_id()
    }

    /// Process an event and return the app ID to switch to, if any
    pub fn handle_event(&mut self, event: &AppEvent) -> Option<&'static str> {
        match event {
            AppEvent::Backend(CharonEvent::QMKEvent(QMKEvent::LayerChange(layer))) => {
                self.on_layer_change(*layer)
            }
            AppEvent::Tick(_) => self.check_hesitation_timer(),
            _ => None,
        }
    }

    fn on_layer_change(&mut self, layer: u8) -> Option<&'static str> {
        if layer == 0 {
            // Back to base layer - return to default view
            self.layer_change_at = None;
            self.pending_layer = None;

            if self.active_view != self.default_view {
                self.active_view = self.default_view;
                return Some(self.active_view.app_id());
            }
        } else {
            // Non-base layer - start hesitation timer
            self.layer_change_at = Some(Instant::now());
            self.pending_layer = Some(layer);
        }
        None
    }

    fn check_hesitation_timer(&mut self) -> Option<&'static str> {
        let Some(changed_at) = self.layer_change_at else {
            return None;
        };

        if changed_at.elapsed() >= self.hesitation_duration {
            // Hesitation period passed - show layer preview
            self.layer_change_at = None;

            if self.active_view != PassThroughView::Keymap {
                self.active_view = PassThroughView::Keymap;
                return Some(PassThroughView::Keymap.app_id());
            }
        }
        None
    }

    /// Get the pending layer number (for Keymap to display)
    pub fn pending_layer(&self) -> Option<u8> {
        self.pending_layer
    }

    /// Check if we're currently showing a temporary view (like layer preview)
    pub fn is_temporary_view(&self) -> bool {
        self.active_view != self.default_view
    }

    /// Set the default pass-through view (user preference)
    pub fn set_default_view(&mut self, view: PassThroughView) {
        self.default_view = view;
    }
}
