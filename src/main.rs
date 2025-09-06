use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::SystemServices::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::LibraryLoader::*;
use std::mem::size_of;
//#[derive(Debug, Copy, Clone)]
//struct Rgb {
//    r: u8,
//    g: u8,
//    b: u8,
//}
//
//impl Rgb {
//    fn new(r: u8, g: u8, b: u8) -> Self {
//        Rgb { r, g, b }
//    }
//}


const CIRCLE_RADIUS: i32 = 40; // 圆形孔的半径
const WINDOW_WIDTH: i32 = 700;
const WINDOW_HEIGHT: i32 = 500;

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(PCWSTR::null())?;
        let hinstance = HINSTANCE(instance.0);
        let class_name = w!("TransparentHoleWindow");

        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_procedure),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: LoadIconW(None, IDI_APPLICATION)?,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: GetSysColorBrush(COLOR_WINDOW), 
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: LoadIconW(None, IDI_APPLICATION)?,
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST, // 移除 WS_EX_TRANSPARENT
            class_name,
            w!("防止小孩误食"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE, 
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            None,
            None,
            hinstance,
            None,
        );

        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            (screen_width - WINDOW_WIDTH) / 2,
            (screen_height - WINDOW_HEIGHT) / 2,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            SWP_SHOWWINDOW,
        );

        create_transparent_hole(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

unsafe fn create_transparent_hole(hwnd: HWND) {
    let window_region = CreateRectRgn(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);

    let center_x = WINDOW_WIDTH / 2;
    let center_y = WINDOW_HEIGHT / 2;
    let hole_region = CreateEllipticRgn(
        center_x - CIRCLE_RADIUS,
        center_y - CIRCLE_RADIUS,
        center_x + CIRCLE_RADIUS,
        center_y + CIRCLE_RADIUS
    );

    let result = CombineRgn(
        window_region,
        window_region,
        hole_region,
        RGN_DIFF 
    );

    if result != RGN_ERROR && !window_region.is_invalid() {
        SetWindowRgn(hwnd, window_region, true);
    }

    if !hole_region.is_invalid() {
        DeleteObject(hole_region);
    }
    if !window_region.is_invalid() {
        DeleteObject(window_region);
    }

    SetLayeredWindowAttributes(
        hwnd,
        COLORREF(0),
        255, 
        LWA_ALPHA
    );
}

unsafe fn update_window_region(hwnd: HWND) {
    let mut rect = RECT::default();
    GetClientRect(hwnd, &mut rect);

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    create_transparent_hole(hwnd);

    InvalidateRect(hwnd, None, true);
}

unsafe extern "system" fn window_procedure(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut rect = RECT::default();
            GetClientRect(hwnd, &mut rect);

            let background_brush = GetSysColorBrush(COLOR_3DFACE);
            FillRect(hdc, &rect, background_brush);

            let title = "该孔用来防止小孩误食。";
            let title_wide = string_to_wstring(title);
            TextOutW(
                hdc,
                20,
                20,
                title_wide.as_slice() 
            );
            let center_x = (rect.right - rect.left) / 2;
            let center_y = (rect.bottom - rect.top) / 2;

            let pen = CreatePen(PS_DASH, 1, COLORREF(0x00000000));
            SelectObject(hdc, HGDIOBJ(pen.0));
            SelectObject(hdc, GetStockObject(NULL_BRUSH));

            Ellipse(
                hdc,
                center_x - CIRCLE_RADIUS,
                center_y - CIRCLE_RADIUS,
                center_x + CIRCLE_RADIUS,
                center_y + CIRCLE_RADIUS,
            );

            DeleteObject(HGDIOBJ(pen.0));
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_SIZE => {
            update_window_region(hwnd);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, WPARAM(HTCAPTION as _), lparam);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn string_to_wstring(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(s)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect()
}