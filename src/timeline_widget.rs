use egui::{Response, Ui, Widget};

pub struct Timeline {
    pub playhead_position: f32,   // Position of the playhead (normalized 0.0 to 1.0)
    pub regions: Vec<(f32, f32)>, // List of (start, end) for regions, normalized
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            playhead_position: 0.0,
            regions: vec![],
        }
    }
}

// Implement the custom widget
impl Widget for &mut Timeline {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(500.0, 100.0), egui::Sense::click());

        // Draw the timeline background
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, egui::Color32::DARK_GRAY);

        // Draw regions
        for &(start, end) in &self.regions {
            let region_start = rect.left() + start * rect.width();
            let region_end = rect.left() + end * rect.width();
            let region_rect = egui::Rect::from_min_max(egui::pos2(region_start, rect.top()), egui::pos2(region_end, rect.bottom()));
            painter.rect_filled(region_rect, 2.0, egui::Color32::LIGHT_BLUE);
        }

        // Draw the playhead
        let playhead_x = rect.left() + self.playhead_position * rect.width();
        painter.line_segment([egui::pos2(playhead_x, rect.top()), egui::pos2(playhead_x, rect.bottom())], (2.0, egui::Color32::RED));

        // Handle input
        if response.clicked() {
            if let Some(pos) = response.hover_pos() {
                self.playhead_position = (pos.x - rect.left()) / rect.width();
            }
        }

        response
    }
}
