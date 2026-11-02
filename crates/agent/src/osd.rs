//! On-Screen Display (OSD) overlay banner HUD for mode and profile switching (M15.1).
//!
//! Renders a translucent, click-through, non-focus-stealing Windows 11-style HUD banner
//! on the primary display whenever:
//! - Top-row mode is toggled (Fn + Esc: Media keys vs F-keys)
//! - Active profile is switched
//! - Remapping is paused or resumed
//!
//! Uses `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOPMOST` to ensure:
//! - Mouse clicks pass directly through to games/applications below
//! - Typing/caret focus is never stolen from foreground applications
//! - Rendered with rounded corners and acrylic dark translucency

use std::sync::Mutex;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateSolidBrush, DT_CENTER, DT_NOPREFIX, DT_SINGLELINE,
    DeleteObject, DrawTextW, EndPaint, FONT_CHARSET, FONT_CLIP_PRECISION, FONT_OUTPUT_PRECISION,
    FONT_QUALITY, FW_BOLD, FW_NORMAL, HDC, InvalidateRect, PAINTSTRUCT, PS_SOLID, RoundRect,
    SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetSystemMetrics, KillTimer,
    LWA_ALPHA, RegisterClassExW, SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SW_SHOWNOACTIVATE,
    SWP_NOACTIVATE, SWP_SHOWWINDOW, SetLayeredWindowAttributes, SetTimer, SetWindowPos,
    WNDCLASS_STYLES, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::core::{PCWSTR, w};

use magicbridge_core::TopRowMode;

/// Window class name for the OSD overlay window.
const OSD_CLASS_NAME: PCWSTR = w!("MacKeys_OSD_Class");

/// Timer ID for auto-dismissing the OSD window.
const IDT_OSD_HIDE: usize = 201;

/// Shared text buffer for the OSD title and subtitle.
static OSD_TEXT: Mutex<Option<(String, String)>> = Mutex::new(None);

/// Global handle for the single OSD window instance.
static OSD_HWND: Mutex<Option<isize>> = Mutex::new(None);

/// Window procedure for the OSD overlay.
unsafe extern "system" fn osd_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        windows::Win32::UI::WindowsAndMessaging::WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            // SAFETY: BeginPaint prepares hwnd for painting.
            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
            if !hdc.is_invalid() {
                paint_osd(hwnd, hdc);
                // SAFETY: EndPaint releases the device context.
                unsafe {
                    let _ = EndPaint(hwnd, &ps);
                }
            }
            LRESULT(0)
        }
        windows::Win32::UI::WindowsAndMessaging::WM_TIMER => {
            if wparam.0 == IDT_OSD_HIDE {
                // SAFETY: KillTimer stops the hide timer.
                unsafe {
                    let _ = KillTimer(Some(hwnd), IDT_OSD_HIDE);
                    let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd, SW_HIDE);
                }
            }
            LRESULT(0)
        }
        _ => {
            // SAFETY: Delegate unhandled messages to DefWindowProcW.
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }
}

/// Paints the translucent rounded acrylic HUD banner onto the OSD window.
fn paint_osd(hwnd: HWND, hdc: HDC) {
    let mut rect = RECT::default();
    // SAFETY: GetClientRect retrieves client bounds of hwnd.
    if unsafe { GetClientRect(hwnd, &mut rect) }.is_err() {
        return;
    }

    let (title, subtitle) = OSD_TEXT
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_else(|| ("MacKeys".to_string(), String::new()));

    // 1. Dark Acrylic background: #14181E
    // SAFETY: CreateSolidBrush creates background fill.
    let bg_brush = unsafe { CreateSolidBrush(COLORREF(0x001E1814)) };
    // 2. Cyan accent border: #00D2FF
    // SAFETY: CreatePen creates accent border.
    let border_pen = unsafe { CreatePen(PS_SOLID, 2, COLORREF(0x00FFD200)) };

    // SAFETY: Select objects into HDC and draw rounded rectangle.
    unsafe {
        let old_brush = SelectObject(hdc, bg_brush.into());
        let old_pen = SelectObject(hdc, border_pen.into());

        let _ = RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 20, 20);

        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(bg_brush.into());
        let _ = DeleteObject(border_pen.into());

        let _ = SetBkMode(hdc, TRANSPARENT);
    }

    // 3. Draw Title (Font: Segoe UI, 20px, Bold, White)
    // SAFETY: CreateFontW creates font descriptor.
    let title_font = unsafe {
        CreateFontW(
            20,
            0,
            0,
            0,
            FW_BOLD.0 as i32,
            0,
            0,
            0,
            FONT_CHARSET(0),
            FONT_OUTPUT_PRECISION(0),
            FONT_CLIP_PRECISION(0),
            FONT_QUALITY(0),
            0,
            w!("Segoe UI"),
        )
    };

    if !title_font.is_invalid() {
        // SAFETY: Select font and draw title text.
        unsafe {
            let old_font = SelectObject(hdc, title_font.into());
            let _ = SetTextColor(hdc, COLORREF(0x00FFFFFF));

            let mut title_rect = RECT {
                left: rect.left,
                top: rect.top + 10,
                right: rect.right,
                bottom: rect.top + 34,
            };
            let mut title_u16: Vec<u16> = title.encode_utf16().collect();
            let _ = DrawTextW(
                hdc,
                &mut title_u16,
                &mut title_rect,
                DT_CENTER | DT_SINGLELINE | DT_NOPREFIX,
            );

            SelectObject(hdc, old_font);
            let _ = DeleteObject(title_font.into());
        }
    }

    // 4. Draw Subtitle (Font: Segoe UI, 13px, Normal, Light Gray #C8D0D8)
    if !subtitle.is_empty() {
        // SAFETY: CreateFontW creates subtitle font.
        let sub_font = unsafe {
            CreateFontW(
                13,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                0,
                0,
                0,
                FONT_CHARSET(0),
                FONT_OUTPUT_PRECISION(0),
                FONT_CLIP_PRECISION(0),
                FONT_QUALITY(0),
                0,
                w!("Segoe UI"),
            )
        };

        if !sub_font.is_invalid() {
            // SAFETY: Select font and draw subtitle text.
            unsafe {
                let old_font = SelectObject(hdc, sub_font.into());
                let _ = SetTextColor(hdc, COLORREF(0x00D8D0C8));

                let mut sub_rect = RECT {
                    left: rect.left,
                    top: rect.top + 36,
                    right: rect.right,
                    bottom: rect.bottom - 10,
                };
                let mut sub_u16: Vec<u16> = subtitle.encode_utf16().collect();
                let _ = DrawTextW(
                    hdc,
                    &mut sub_u16,
                    &mut sub_rect,
                    DT_CENTER | DT_SINGLELINE | DT_NOPREFIX,
                );

                SelectObject(hdc, old_font);
                let _ = DeleteObject(sub_font.into());
            }
        }
    }
}

/// On-Screen Display HUD controller.
pub struct OsdOverlay {
    hwnd: HWND,
}

impl OsdOverlay {
    /// Creates and registers the OSD overlay window.
    pub fn new() -> Result<Self, windows::core::Error> {
        // SAFETY: GetModuleHandleW retrieves instance handle.
        let hinstance = unsafe { GetModuleHandleW(None)? };

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: WNDCLASS_STYLES(0),
            lpfnWndProc: Some(osd_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance.into(),
            hIcon: windows::Win32::UI::WindowsAndMessaging::HICON::default(),
            hCursor: windows::Win32::UI::WindowsAndMessaging::HCURSOR::default(),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: OSD_CLASS_NAME,
            hIconSm: windows::Win32::UI::WindowsAndMessaging::HICON::default(),
        };

        // SAFETY: RegisterClassExW registers class; ignore already-exists error.
        unsafe {
            let _ = RegisterClassExW(&wnd_class);
        }

        // Layered, click-through, non-focus-stealing topmost popup
        let ex_style =
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_TOOLWINDOW;

        // SAFETY: CreateWindowExW creates the OSD window with 0 size initially.
        let hwnd = unsafe {
            CreateWindowExW(
                ex_style,
                OSD_CLASS_NAME,
                w!("MacKeys OSD"),
                WS_POPUP,
                0,
                0,
                380,
                68,
                None,
                None,
                Some(hinstance.into()),
                None,
            )?
        };

        // Set opacity to 235 / 255 (translucent acrylic look)
        // SAFETY: SetLayeredWindowAttributes sets window alpha.
        unsafe {
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 235, LWA_ALPHA);
        }

        if let Ok(mut g) = OSD_HWND.lock() {
            *g = Some(hwnd.0 as isize);
        }

        Ok(Self { hwnd })
    }

    /// Displays the OSD banner with `title` and `subtitle` for `duration_ms`.
    pub fn show(&self, title: impl Into<String>, subtitle: impl Into<String>, duration_ms: u32) {
        if let Ok(mut g) = OSD_TEXT.lock() {
            *g = Some((title.into(), subtitle.into()));
        }

        // Compute primary monitor center near the bottom
        // SAFETY: GetSystemMetrics queries display dimensions.
        let (screen_w, screen_h) =
            unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };

        let osd_w = 400;
        let osd_h = 68;
        let x = (screen_w - osd_w) / 2;
        let y = screen_h - 130; // 60-80px above taskbar

        // SAFETY: Reposition, redraw, and show without activating focus.
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                Some(windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST),
                x,
                y,
                osd_w,
                osd_h,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            let _ = InvalidateRect(Some(self.hwnd), None, true);
            let _ =
                windows::Win32::UI::WindowsAndMessaging::ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            // Set auto-dismiss timer
            let _ = SetTimer(Some(self.hwnd), IDT_OSD_HIDE, duration_ms, None);
        }
    }

    /// Displays Top-Row Mode switch HUD.
    pub fn show_top_row_mode(&self, mode: TopRowMode, duration_ms: u32) {
        match mode {
            TopRowMode::MediaKeysFirst => {
                self.show(
                    "⚡ Media Keys Mode",
                    "F7–F12 control volume/playback. Hold Fn for F-keys.",
                    duration_ms,
                );
            }
            TopRowMode::FKeysFirst => {
                self.show(
                    "⚡ Function Keys Mode",
                    "F1–F12 send standard Windows keys. Hold Fn for media.",
                    duration_ms,
                );
            }
        }
    }

    /// Displays Profile change HUD.
    pub fn show_profile(&self, profile_name: &str, duration_ms: u32) {
        self.show(
            format!("⚡ Profile: {profile_name}"),
            "Keyboard mapping profile activated",
            duration_ms,
        );
    }

    /// Displays Remapping Paused / Active HUD.
    pub fn show_status(&self, enabled: bool, game_mode: bool, duration_ms: u32) {
        if !enabled {
            self.show(
                "⏸ Remapping Paused",
                "Apple keys are passing through unmodified",
                duration_ms,
            );
        } else if game_mode {
            self.show(
                "🎮 Game Mode Active",
                "Remapping auto-paused for fullscreen application",
                duration_ms,
            );
        } else {
            self.show(
                "▶ Remapping Active",
                "Apple keys are mapped to Windows layout",
                duration_ms,
            );
        }
    }
}

impl Drop for OsdOverlay {
    fn drop(&mut self) {
        if let Ok(mut g) = OSD_HWND.lock() {
            *g = None;
        }
        // SAFETY: DestroyWindow cleans up the OSD window.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osd_text_mutex() {
        let mut g = OSD_TEXT.lock().unwrap();
        *g = Some(("Test".to_string(), "Sub".to_string()));
        assert!(g.is_some());
    }
}
