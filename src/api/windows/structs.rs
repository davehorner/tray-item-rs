use std::sync::mpsc::Sender;

use windows_sys::Win32::{
    Foundation::{HMODULE, HWND},
    UI::WindowsAndMessaging::HMENU,
};

#[derive(Clone)]
pub(crate) struct WindowInfo {
    pub hwnd: HWND,
    pub hmodule: HMODULE,
    pub hmenu: HMENU,
    pub tray_leftclick: u32,
    pub tray_rightclick: u32,
}

unsafe impl Send for WindowInfo {}
unsafe impl Sync for WindowInfo {}

#[derive(Clone)]
pub(crate) struct WindowsLoopData {
    pub info: WindowInfo,
    pub tx: Sender<WindowsTrayEvent>,
}

pub(crate) struct WindowsTrayEvent(pub(crate) u32);
