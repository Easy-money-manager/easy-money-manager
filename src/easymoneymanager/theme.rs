use super::EasyMoneyManager;

use eframe::egui;
use egui::{ TextStyle, FontId };


#[allow(dead_code)]
pub(super) struct Theme;
impl Theme {
    const BACKGROUND: egui::Color32 =
        egui::Color32::from_rgb(10, 12, 11);

    const CARD: egui::Color32 =
        egui::Color32::from_rgb(18, 22, 20);

    const CARD_HOVER: egui::Color32 =
        egui::Color32::from_rgb(24, 30, 27);

    const TEXT: egui::Color32 =
        egui::Color32::from_rgb(245, 245, 245);

    const MUTED: egui::Color32 =
        egui::Color32::from_rgb(170, 170, 170);

    const GOLD: egui::Color32 =
        egui::Color32::from_rgb(220, 180, 80);

    const GREEN: egui::Color32 =
        egui::Color32::from_rgb(30, 160, 100);

    fn border() -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)
    }

    fn gold_border() -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(220, 180, 80, 90)
    }
}

impl EasyMoneyManager {
    pub(super) fn configure_style(ctx: &egui::Context) {
        let mut style: egui::Style = (*ctx.style_of(egui::Theme::Dark)).clone();

        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::proportional(30.0),
            ),
            (
                TextStyle::Body,
                FontId::proportional(20.0),
            ),
            (
                TextStyle::Button,
                FontId::proportional(19.0),
            ),
            (
                TextStyle::Small,
                FontId::proportional(16.0),
            ),
            (
                TextStyle::Monospace,
                FontId::monospace(18.0),
            ),
            ]
                .into();
        style.visuals.widgets.inactive.weak_bg_fill = Theme::CARD;
        style.visuals.widgets.inactive.bg_fill = Theme::CARD;
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Theme::border());
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, Theme::gold_border());
        style.visuals.widgets.hovered.weak_bg_fill = Theme::CARD_HOVER;
        style.visuals.widgets.hovered.bg_fill = Theme::CARD_HOVER;
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, Theme::GOLD);
        style.visuals.widgets.active.weak_bg_fill = Theme::CARD_HOVER;
        style.visuals.widgets.active.bg_fill = Theme::CARD_HOVER;
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, Theme::GOLD);
        style.spacing.item_spacing = egui::vec2(14.0, 12.0);
        style.spacing.button_padding = egui::vec2(18.0, 12.0);
        style.spacing.interact_size = egui::vec2(48.0, 34.0);
        style.visuals.widgets.open.weak_bg_fill = Theme::CARD_HOVER;
        style.visuals.widgets.open.bg_fill = Theme::CARD_HOVER;
        style.visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, Theme::GOLD);
        style.spacing.interact_size = egui::vec2(44.0, 36.0);
        let background = egui::Color32::from_rgb(10, 12, 11);      // #0a0c0b
        let card = egui::Color32::from_rgb(18, 22, 20);            // #121614
        let card_hover = egui::Color32::from_rgb(24, 30, 27);      // #181e1b

        let text = egui::Color32::from_rgb(245, 245, 245);          // #f5f5f5
        let muted = egui::Color32::from_rgb(170, 170, 170);         // #aaa

        let border = egui::Color32::from_rgba_unmultiplied(
            255, 255, 255, 20
        );

        let gold = egui::Color32::from_rgb(220, 180, 80);
        let green = egui::Color32::from_rgb(30, 160, 100);

        style.visuals = egui::Visuals::dark();

        style.visuals.panel_fill = background;
        style.visuals.window_fill = card;
        style.visuals.extreme_bg_color = background;

        style.visuals.override_text_color = Some(text);

        style.visuals.widgets.inactive.bg_fill = card;
        style.visuals.widgets.inactive.weak_bg_fill = card;
        style.visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0, border);
        style.visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0, text);

        style.visuals.widgets.hovered.bg_fill = card_hover;
        style.visuals.widgets.hovered.bg_stroke =
            egui::Stroke::new(1.0, gold);
        style.visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0, text);

        style.visuals.widgets.active.bg_fill = card_hover;
        style.visuals.widgets.active.bg_stroke =
            egui::Stroke::new(1.0, gold);
        style.visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0, text);

        style.visuals.selection.bg_fill = green;
        style.visuals.selection.stroke =
            egui::Stroke::new(1.0, text);

        style.visuals.widgets.inactive.corner_radius =
            egui::CornerRadius::same(14);

        style.visuals.widgets.hovered.corner_radius =
            egui::CornerRadius::same(14);

        style.visuals.widgets.active.corner_radius =
            egui::CornerRadius::same(14);

        ctx.set_style_of(egui::Theme::Dark, style);
    }
    fn glow(
        painter: &egui::Painter,
        center: egui::Pos2,
        radius: f32,
        color: egui::Color32,
    ) {
        let steps: usize = 24;

        for i in (1..=steps).rev() {
            let factor = i as f32 / steps as f32;

            let r = radius * factor;

            let alpha = (
                color.a() as f32
                * (1.0 - factor).powf(1.5)
            ) as u8;

            let glow_color = egui::Color32::from_rgba_unmultiplied(
                color.r(),
                color.g(),
                color.b(),
                alpha,
            );

            painter.circle_filled(
                center,
                r,
                glow_color,
            );
        }
    }
    fn paint_grid(
        painter: &egui::Painter,
        rect: egui::Rect,
    ) {
        let spacing: f32 = 48.0;

        let color = egui::Color32::from_rgba_unmultiplied(
            255, 255, 255, 8
        );

        let stroke = egui::Stroke::new(1.0, color);

        let mut x = rect.left();

        while x < rect.right() {
            painter.line_segment(
                [
                egui::pos2(x, rect.top()),
                egui::pos2(x, rect.bottom()),
                ],
                stroke,
            );

            x += spacing;
        }

        let mut y = rect.top();

        while y < rect.bottom() {
            painter.line_segment(
                [
                egui::pos2(rect.left(), y),
                egui::pos2(rect.right(), y),
                ],
                stroke,
            );

            y += spacing;
        }
    }
    fn paint_streaks(
        painter: &egui::Painter,
        rect: egui::Rect,
    ) {
        let stroke = egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(
                220, 180, 80, 10
            ),
        );

        let spacing: f32 = 220.0;

        let mut x = rect.left() - rect.height();

        while x < rect.right() {
            painter.line_segment(
                [
                egui::pos2(x, rect.bottom()),
                egui::pos2(
                    x + rect.height(),
                    rect.top(),
                ),
                ],
                stroke,
            );

            x += spacing;
        }
    }
    pub(super) fn paint_background(ui: &mut egui::Ui) {
        let rect: egui::Rect = ui.max_rect();
        let painter: &egui::Painter = ui.painter();

        painter.rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgb(7, 9, 8),
        );

        Self::glow(
            &painter,
            rect.left_top() + egui::vec2(260.0, 180.0),
            420.0,
            egui::Color32::from_rgba_unmultiplied(
                20, 180, 100, 80
            ),
        );

        Self::glow(
            &painter,
            rect.right_top() + egui::vec2(-300.0, 170.0),
            360.0,
            egui::Color32::from_rgba_unmultiplied(
                220, 170, 60, 65
            ),
        );

        Self::glow(
            &painter,
            rect.center_bottom() + egui::vec2(0.0, -120.0),
            500.0,
            egui::Color32::from_rgba_unmultiplied(
                0, 120, 90, 55
            ),
        );

        Self::paint_grid(&painter, rect);
        Self::paint_streaks(&painter, rect);
    }
}
