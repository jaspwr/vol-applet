use std::{ ffi::c_void, mem, sync::Mutex };

use gdk_sys::GdkRectangle;
use gobject_sys::{ g_signal_connect_data, GCallback, GObject };
use gtk::{
    gdk_pixbuf::Pixbuf,
    glib::{ ffi::gpointer, idle_add_once, translate::ToGlibPtr },
    traits::IconThemeExt,
    IconLookupFlags,
};
use gtk_sys::*;

use crate::{ exception::Exception, popout::Popout, audio::shared_output_list, AUDIO, elements::Percentise };

static TRAY_ICON: Mutex<Option<TrayIcon>> = Mutex::new(None);

pub struct TrayIcon {
    pub icon_ptr: *mut gtk_sys::GtkStatusIcon,
    level: VolumeLevel,
    volume: f32,
    muted: bool,
}
unsafe impl Sync for TrayIcon {}
unsafe impl Send for TrayIcon {}

impl TrayIcon {
    fn fetch_icon(icon_name: &str) -> Option<Pixbuf> {
        let theme = gtk::IconTheme::default()?;
        let flags = IconLookupFlags::empty();
        let icon = theme.lookup_icon(icon_name, 16, flags)?;
        match icon.load_icon() {
            Ok(icon_pix) => Some(icon_pix),
            Err(_) => None,
        }
    }

    fn create_icon(&mut self) {
        let icon_pix = Self::fetch_icon(self.level.to_icon()).unwrap();

        unsafe {
            self.icon_ptr = gtk_status_icon_new_from_pixbuf(icon_pix.to_glib_none().0);
            gtk_status_icon_set_visible(self.icon_ptr, 1);

            g_signal_connect(
                self.icon_ptr as *mut c_void,
                "activate".to_glib_none().0,
                Some(mem::transmute(activate_cb as *const ())),
                std::ptr::null_mut()
            );

            g_signal_connect(
                self.icon_ptr as *mut c_void,
                "popup-menu".to_glib_none().0,
                Some(mem::transmute(popup_cb as *const ())),
                std::ptr::null_mut()
            );
        }
        AUDIO.lock()
            .unwrap()
            .aud.get_outputs(
                Box::new(|outputs: Vec<shared_output_list::Output>| {
                    for output in outputs {
                        if output.is_default() {
                            TrayIcon::set_volume(output.volume);
                            TrayIcon::set_muted(output.muted);
                        }
                    }
                })
            );
    }

    pub fn set_volume(volume: f32) {
        if let Some(icon) = TRAY_ICON.lock().unwrap().as_mut() {
            icon.volume = volume;
        }

        idle_add_once(move || {
            if let Some(icon) = TRAY_ICON.lock().unwrap().as_mut() {
                if let Err(e) = icon.set_volume_icon_level(volume, icon.muted) {
                    e.log_and_exit();
                }
            }
        });
    }

    pub fn set_tooltip_volume(volume: f32) {
        idle_add_once(move || {
            if let Some(icon) = TRAY_ICON.lock().unwrap().as_mut() {
                let tooltip = volume.format_volume();
                unsafe {
                    gtk_status_icon_set_tooltip_text(icon.icon_ptr, tooltip.as_str().to_glib_none().0);
                }
            }
        });
    }

    fn set_volume_icon_level(&mut self, volume: f32, muted: bool) -> Result<(), Exception> {
        let new_lvl = VolumeLevel::from_volume(volume, muted);
        Self::set_tooltip_volume(volume);
        if self.level == new_lvl {
            return Ok(());
        }
        self.level = new_lvl;
        match Self::fetch_icon(self.level.to_icon()) {
            Some(icon_pix) => {
                self.set_icon(icon_pix);
                Ok(())
            }
            None => Err(Exception::Misc("Could not find icon".to_string())),
        }
    }

    pub fn get_geometry() -> (GdkRectangle, GtkOrientation) {
        let icon_ptr = TRAY_ICON.lock().unwrap().as_mut().unwrap().icon_ptr;

        let area = Box::new(GdkRectangle::default());
        let orient = Box::<GtkOrientation>::new(GtkOrientation::MAX);
        unsafe {
            let area_ptr = Box::into_raw(area);
            let orient_ptr = Box::into_raw(orient);
            gtk_status_icon_get_geometry(icon_ptr, std::ptr::null_mut(), area_ptr, orient_ptr);
            #[allow(clippy::clone_on_copy)]
            ((*area_ptr).clone(), (*orient_ptr).clone())
        }
    }

    fn set_icon(&self, icon_pixbuf: Pixbuf) {
        unsafe {
            gtk_status_icon_set_from_pixbuf(self.icon_ptr, icon_pixbuf.to_glib_none().0);
        }
    }

    pub fn set_muted(muted: bool) {
        let mut vol = 0.;
        if let Some(icon) = TRAY_ICON.lock().unwrap().as_mut() {
            icon.muted = muted;
            vol = icon.volume;
        }
        TrayIcon::set_volume(vol);
    }

    pub fn initialise() {
        let mut tray_icon = Self {
            icon_ptr: std::ptr::null_mut(),
            level: VolumeLevel::High,
            volume: 0.,
            muted: false,
        };
        tray_icon.create_icon();
        TRAY_ICON.lock().unwrap().replace(tray_icon);
    }
}

#[derive(PartialEq)]
enum VolumeLevel {
    High,
    Medium,
    Low,
    Muted,
}

impl VolumeLevel {
    fn from_volume(volume: f32, muted: bool) -> VolumeLevel {
        if muted {
            return VolumeLevel::Muted;
        }

        if volume > 66. {
            VolumeLevel::High
        } else if volume > 33. {
            VolumeLevel::Medium
        } else {
            VolumeLevel::Low
        }
    }

    fn to_icon(&self) -> &'static str {
        match self {
            VolumeLevel::High => VOLUME_HIGH,
            VolumeLevel::Medium => VOLUME_MEDIUM,
            VolumeLevel::Low => VOLUME_LOW,
            VolumeLevel::Muted => VOLUME_MUTED,
        }
    }
}

#[no_mangle]
extern "C" fn activate_cb(_: gpointer, _: gpointer) {
    Popout::show();
}

#[no_mangle]
extern "C" fn popup_cb(_: gpointer, _: gpointer) {
    Popout::show();
}

unsafe fn g_signal_connect(
    instance: gpointer,
    detailed_signal: *const i8,
    c_handler: GCallback,
    data: gpointer
) -> u64 {
    g_signal_connect_data(
        instance as *mut GObject,
        detailed_signal,
        c_handler,
        data,
        None,
        std::mem::transmute(0)
    )
}

static VOLUME_HIGH: &str = "audio-volume-high-symbolic";
static VOLUME_MEDIUM: &str = "audio-volume-medium-symbolic";
static VOLUME_LOW: &str = "audio-volume-low-symbolic";
static VOLUME_MUTED: &str = "audio-volume-muted-symbolic";

trait DefaultRect {
    fn default() -> Self;
}

impl DefaultRect for GdkRectangle {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}
