use std::rc::Rc;

use gtk::{traits::{ProgressBarExt, ContainerExt, RangeExt, ButtonExt}, prelude::ObjectExt, glib, ProgressBar};

type ReRunTimer = Option<std::time::Duration>;

enum Text {
    Static(String),
    Command(String, ReRunTimer),
}

pub struct VolumeSlider {
    label: Option<String>,
    bar: ProgressBar
}

impl VolumeSlider {
    pub fn new(container: &gtk::Box, label: Option<String>, start_value: f64, muted: bool, 
        on_change_vol: Rc<dyn Fn(f64) -> () + 'static>,
        on_change_mute: Rc<dyn Fn(bool) -> () + 'static>) -> VolumeSlider {

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


        let slider = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        slider.set_value(start_value);
        slider.connect_change_value(move |_, _, d: f64| -> glib::signal::Inhibit {
            on_change_vol(d);
            gtk::Inhibit(false)
        });

        let mute_icon = if muted {
            "audio-volume-muted"
        } else {
            "audio-volume-high"
        };

        let mute_button = gtk::Button::from_icon_name(Some(mute_icon), gtk::IconSize::Button);
        mute_button.connect_clicked(move |_| {
            on_change_mute(!muted);
        });

        container.add(&slider);
        // container.add(&mute_button);
        // container.add(&bar);

        VolumeSlider {
            label,
            bar
        }
    }

    pub fn set_bar(&mut self, value: f64) {
        self.bar.set_fraction(value);
    }
}