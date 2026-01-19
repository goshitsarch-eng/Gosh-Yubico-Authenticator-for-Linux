use gtk4::gdk;
use gtk4::glib;
use gtk4::graphene;
use gtk4::gsk;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::Cell;
use std::f32::consts::PI;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct CountdownIndicator {
        /// Progress from 0.0 (empty) to 1.0 (full)
        pub progress: Cell<f64>,
        /// Remaining seconds to display
        pub remaining: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CountdownIndicator {
        const NAME: &'static str = "GoshCountdownIndicator";
        type Type = super::CountdownIndicator;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for CountdownIndicator {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecDouble::builder("progress")
                        .minimum(0.0)
                        .maximum(1.0)
                        .default_value(1.0)
                        .build(),
                    glib::ParamSpecUInt::builder("remaining")
                        .minimum(0)
                        .maximum(60)
                        .default_value(30)
                        .build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "progress" => {
                    self.progress.set(value.get().unwrap());
                    self.obj().queue_draw();
                }
                "remaining" => {
                    self.remaining.set(value.get().unwrap());
                    self.obj().queue_draw();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "progress" => self.progress.get().to_value(),
                "remaining" => self.remaining.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for CountdownIndicator {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;
            let size = width.min(height);
            let center_x = width / 2.0;
            let center_y = height / 2.0;
            let radius = (size / 2.0) - 2.0;
            let line_width = 3.0f32;
            let progress = self.progress.get() as f32;

            // Get colors from theme
            let style_context = widget.style_context();
            let accent_color = style_context
                .lookup_color("accent_bg_color")
                .unwrap_or(gdk::RGBA::new(0.2, 0.5, 0.8, 1.0));
            let bg_color = style_context
                .lookup_color("view_bg_color")
                .unwrap_or(gdk::RGBA::new(0.9, 0.9, 0.9, 0.3));

            // Draw background circle
            let bg_circle_color = gdk::RGBA::new(
                bg_color.red(),
                bg_color.green(),
                bg_color.blue(),
                0.3,
            );
            snapshot.append_border(
                &gsk::RoundedRect::from_rect(
                    graphene::Rect::new(
                        center_x - radius,
                        center_y - radius,
                        radius * 2.0,
                        radius * 2.0,
                    ),
                    radius,
                ),
                &[line_width; 4],
                &[bg_circle_color; 4],
            );

            // Draw progress arc using Cairo
            let cr = snapshot.append_cairo(&graphene::Rect::new(0.0, 0.0, width, height));

            // Set the accent color
            cr.set_source_rgba(
                accent_color.red() as f64,
                accent_color.green() as f64,
                accent_color.blue() as f64,
                accent_color.alpha() as f64,
            );
            cr.set_line_width(line_width as f64);
            cr.set_line_cap(::cairo::LineCap::Round);

            // Draw arc from top (start at -90 degrees), going clockwise
            let start_angle = -PI / 2.0;
            let end_angle = start_angle + (2.0 * PI * progress);

            cr.arc(
                center_x as f64,
                center_y as f64,
                (radius - line_width / 2.0) as f64,
                start_angle as f64,
                end_angle as f64,
            );
            let _ = cr.stroke();

            // Draw remaining seconds in center
            let remaining = self.remaining.get();
            if remaining > 0 {
                let text = remaining.to_string();
                let text_color = style_context
                    .lookup_color("window_fg_color")
                    .unwrap_or(gdk::RGBA::new(0.0, 0.0, 0.0, 1.0));

                cr.set_source_rgba(
                    text_color.red() as f64,
                    text_color.green() as f64,
                    text_color.blue() as f64,
                    text_color.alpha() as f64,
                );
                cr.set_font_size(11.0);

                let extents = cr.text_extents(&text).unwrap();
                cr.move_to(
                    center_x as f64 - extents.width() / 2.0,
                    center_y as f64 + extents.height() / 2.0,
                );
                let _ = cr.show_text(&text);
            }
        }

        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            // Request a square widget
            let size = 32;
            (size, size, -1, -1)
        }
    }
}

glib::wrapper! {
    pub struct CountdownIndicator(ObjectSubclass<imp::CountdownIndicator>)
        @extends gtk4::Widget;
}

impl CountdownIndicator {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_progress(&self, progress: f64) {
        self.set_property("progress", progress);
    }

    pub fn set_remaining(&self, remaining: u32) {
        self.set_property("remaining", remaining);
    }

    pub fn update(&self, remaining: u32, progress: f64) {
        self.imp().remaining.set(remaining);
        self.imp().progress.set(progress);
        self.queue_draw();
    }
}

impl Default for CountdownIndicator {
    fn default() -> Self {
        Self::new()
    }
}
