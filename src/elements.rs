use std::rc::Rc;

use gtk::{traits::{ProgressBarExt, ContainerExt, RangeExt, ButtonExt, GridExt}, glib, ProgressBar};

unsafe impl Sync for VolumeSlider {}
unsafe impl Send for VolumeSlider {}
pub struct VolumeSlider {
    label: Option<String>,
    volume_slider: gtk::Scale,
    mute_button: gtk::Button,
    bar: ProgressBar
}

impl VolumeSlider {
    pub fn new(container: &gtk::Box, label: Option<String>, start_value: f32, muted: bool, 
        on_change_vol: Rc<dyn Fn(f32) + 'static>,
        on_change_mute: Rc<dyn Fn() + 'static>) -> VolumeSlider {

        if let Some(label_text) = &label {
            let label = gtk::Label::builder()
                .label(&label_text.to_owned())
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .build();
            container.add(&label);
        }

        let bar = gtk::ProgressBar::new();
        bar.set_fraction(0.);


        let volume_slider = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        volume_slider.set_value(start_value as f64);
        volume_slider.connect_change_value(move |_, _, d: f64| -> glib::signal::Inhibit {
            on_change_vol(d as f32);
            gtk::Inhibit(false)
        });

        let mute_icon = mute_button_icon(muted);

        let mute_button = gtk::Button::from_icon_name(Some(mute_icon), gtk::IconSize::Button);
        mute_button.connect_clicked(move |_| {
            on_change_mute();
        });

        let grid: gtk::Grid = gtk::Grid::new();
        grid.set_column_spacing(10);
        grid.attach(&volume_slider, 0, 0, 30, 3);
        grid.attach_next_to(&mute_button, Some(&volume_slider), gtk::PositionType::Right, 3, 3);
        // container.add(&bar);
        container.add(&grid);

        VolumeSlider {
            label,
            volume_slider,
            mute_button,
            bar
        }
    }

    pub fn set_volume_slider(&self, value: f32) {
        self.volume_slider.set_value(value as f64);
    }

    pub fn set_muted(&self, muted: bool) {
        let icon = mute_button_icon(muted);
        self.mute_button.set_image(Some(&gtk::Image::from_icon_name(Some(icon), gtk::IconSize::Button)));
    }

    pub fn set_bar(&mut self, value: f32) {
        self.bar.set_fraction(value as f64);
    }
}

fn mute_button_icon(muted: bool) -> &'static str {
    if muted {
        "audio-volume-muted"
    } else {
        "audio-volume-high"
    }
}