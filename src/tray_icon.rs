use std::{mem, ffi::c_void, sync::{Mutex, Arc}};

use gdk_sys::GdkRectangle;
use gobject_sys::{g_signal_connect_data, GCallback, GObject};
use gtk::{gdk_pixbuf::Pixbuf, IconLookupFlags,
    traits::{IconThemeExt, WidgetExt, GtkWindowExt},
    glib::{translate::ToGlibPtr, ffi::gpointer}, gdk::keys::constants::p};
use gtk_sys::*;

use crate::{exception::Exception, popout::{Popout, self}, TRAY_ICON, POPOUT};



pub struct TrayIcon {
    icon_ptr: StatusIconPtr,
    area: GdkRectangle,
    orientation: GtkOrientation,
    level: VolumeLevel
}

unsafe impl Sync for StatusIconPtr {}
unsafe impl Send for StatusIconPtr {}
struct StatusIconPtr {
    ptr: *mut gtk_sys::GtkStatusIcon
}

impl TrayIcon {
    fn fetch_icon(icon_name: &str) -> Option<Pixbuf> {
        let theme = gtk::IconTheme::default()?;
        let flags = IconLookupFlags::empty();
        let icon = theme.lookup_icon(icon_name, 16, flags)?;
        match icon.load_icon() {
            Ok(icon_pix) => Some(icon_pix),
            Err(_) => None
        }
    }

    pub fn create_icon(&mut self, tooltip: &str) {
        let icon_pix = Self::fetch_icon(self.level.to_icon()).unwrap();

        unsafe {
            self.icon_ptr.ptr = gtk_status_icon_new_from_pixbuf(icon_pix.to_glib_none().0);
            gtk_status_icon_set_tooltip_markup(self.icon_ptr.ptr, tooltip.to_glib_none().0);
            gtk_status_icon_set_visible(self.icon_ptr.ptr, 1);
            

            g_signal_connect(
                self.icon_ptr.ptr as *mut c_void, 
                "activate".to_glib_none().0, 
                Some(mem::transmute(status_icon_callback as *const ())),
                std::ptr::null_mut()
            );
        }
    }

    pub fn set_volume_icon_level(&mut self, volume: f32) -> Result<(), Exception> {
        let new_lvl = VolumeLevel::from_volume(volume);
        if self.level == new_lvl { return Ok(()) };
        self.level = new_lvl;
        match Self::fetch_icon(self.level.to_icon()) {
            Some(icon_pix) => {
                self.set_icon(icon_pix);
                Ok(())
            },
            None => Err(Exception::Misc("Could not find icon".to_string()))
        }
    }

    fn set_icon(&self, icon_pixbuf: Pixbuf) {
        unsafe {
            gtk_status_icon_set_from_pixbuf(self.icon_ptr.ptr, icon_pixbuf.to_glib_none().0);
        }
    }

    fn refetch_geometry(&mut self) {
        let area_ptr: *mut GdkRectangle = &mut self.area;
        let orient_ptr: *mut GtkOrientation = &mut self.orientation;
        unsafe {
            gtk_status_icon_get_geometry(
                self.icon_ptr.ptr,
                std::ptr::null_mut(),
                area_ptr,
                orient_ptr,
            );
        }
    }

    pub fn get_geometry(&mut self) -> (GdkRectangle, GtkOrientation) {
        self.refetch_geometry();
        (self.area, self.orientation)
    }

    pub fn align_popout(&mut self) {
        let (area, ori) = self.get_geometry();
        POPOUT.lock().unwrap().as_mut().unwrap().set_geomerty(area, ori);
    }

    pub fn new() -> Self {
        println!("fuck you");
        let mut tray_icon = Self {
            icon_ptr: StatusIconPtr { ptr: std::ptr::null_mut() },
            area: GdkRectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            orientation: 0,
            level: VolumeLevel::High,
        };
        tray_icon.create_icon("Volume");
        tray_icon
    }
}

#[derive(PartialEq)]
enum VolumeLevel {
    High,
    Medium,
    Low,
    Muted
}

impl VolumeLevel {
    fn from_volume(volume: f32) -> VolumeLevel {
        if volume > 0.66 {
            VolumeLevel::High
        } else if volume > 0.33 {
            VolumeLevel::Medium
        } else if volume > 0.0 {
            VolumeLevel::Low
        } else {
            VolumeLevel::Muted
        }
    }

    fn to_icon(&self) -> &'static str {
        const VOLUME_HIGH: &str = "audio-volume-high-symbolic";
        const VOLUME_MEDIUM: &str = "audio-volume-medium-symbolic";
        const VOLUME_LOW: &str = "audio-volume-low-symbolic";
        const VOLUME_MUTED: &str = "audio-volume-muted-symbolic";

        match self {
            VolumeLevel::High => VOLUME_HIGH,
            VolumeLevel::Medium => VOLUME_MEDIUM,
            VolumeLevel::Low => VOLUME_LOW,
            VolumeLevel::Muted => VOLUME_MUTED
        }
    }
}

#[no_mangle]
extern "C" fn status_icon_callback(_: gpointer, _: gpointer) {
    TRAY_ICON.lock().unwrap().as_mut().unwrap().align_popout();
    POPOUT.lock().unwrap().as_mut().unwrap().toggle_vis();
}

unsafe fn g_signal_connect(
    instance: gpointer,
    detailed_signal: *const i8,
    c_handler: GCallback,
    data: gpointer,
) -> u64 {
    g_signal_connect_data(
        instance as *mut GObject,
        detailed_signal,
        c_handler,
        data,
        None,
        std::mem::transmute(0),
    )
}