use tests::windows;
use winrt::Abi;

use windows::win32::backup::{CreateEventW, SetEvent, WaitForSingleObject, RECT};
use windows::win32::base::WM_KEYUP;
use windows::win32::com::CreateUri;
use windows::win32::direct3d12::D3D12_DEFAULT_BLEND_FACTOR_ALPHA;
use windows::win32::direct3d_dxgi::{
    DXGI_ADAPTER_FLAG, DXGI_FORMAT, DXGI_MODE_DESC, DXGI_MODE_SCALING, DXGI_MODE_SCANLINE_ORDER,
    DXGI_RATIONAL,
};
use windows::win32::direct3d_hlsl::D3DCOMPILER_DLL;
use windows::win32::dlg_box::CHOOSECOLORW;
use windows::win32::menu_rc::{PROPENUMPROCA, PROPENUMPROCW};
use windows::win32::security::ACCESS_MODE;
use windows::win32::win_auto::UIA_ScrollPatternNoScroll;
use windows::win32::win_prog::CloseHandle;

#[test]
fn signed_enum32() {
    assert!(ACCESS_MODE::default() == 0.into());
    assert!(ACCESS_MODE::REVOKE_ACCESS.abi() == ACCESS_MODE::REVOKE_ACCESS);
}

#[test]
fn unsigned_enum32() {
    assert!(DXGI_ADAPTER_FLAG::default() == 0.into());
    assert!(
        DXGI_ADAPTER_FLAG::DXGI_ADAPTER_FLAG_SOFTWARE.abi()
            == DXGI_ADAPTER_FLAG::DXGI_ADAPTER_FLAG_SOFTWARE
    );

    let both =
        DXGI_ADAPTER_FLAG::DXGI_ADAPTER_FLAG_SOFTWARE | DXGI_ADAPTER_FLAG::DXGI_ADAPTER_FLAG_REMOTE;
    assert!(both == 3.into());
}

#[test]
fn rect() {
    let rect = RECT {
        left: 1,
        top: 2,
        right: 3,
        bottom: 4,
    };

    assert!(rect.left == 1);
    assert!(rect.top == 2);
    assert!(rect.right == 3);
    assert!(rect.bottom == 4);

    let clone = rect.clone();

    assert!(
        clone
            == RECT {
                left: 1,
                top: 2,
                right: 3,
                bottom: 4,
            }
    );
}

#[test]
fn dxgi_mode_desc() {
    let _ = DXGI_MODE_DESC {
        Width: 1,
        Height: 2,
        RefreshRate: DXGI_RATIONAL {
            Numerator: 3,
            Denominator: 5,
        },
        Format: DXGI_FORMAT::DXGI_FORMAT_R32_TYPELESS,
        ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER::DXGI_MODE_SCANLINE_ORDER_PROGRESSIVE,
        Scaling: DXGI_MODE_SCALING::DXGI_MODE_SCALING_CENTERED,
    };
}

#[cfg(target_pointer_width = "64")]
#[test]
fn size64() {
    assert!(std::mem::size_of::<ACCESS_MODE>() == 4);
    assert!(std::mem::size_of::<DXGI_ADAPTER_FLAG>() == 4);
    assert!(std::mem::size_of::<RECT>() == 16);
    assert!(std::mem::size_of::<DXGI_MODE_DESC>() == 28);
    assert!(std::mem::size_of::<CHOOSECOLORW>() == 72);
}

#[cfg(target_pointer_width = "32")]
#[test]
fn size32() {
    assert!(std::mem::size_of::<ACCESS_MODE>() == 4);
    assert!(std::mem::size_of::<DXGI_ADAPTER_FLAG>() == 4);
    assert!(std::mem::size_of::<RECT>() == 16);
    assert!(std::mem::size_of::<DXGI_MODE_DESC>() == 28);
    assert!(std::mem::size_of::<CHOOSECOLORW>() == 36);
}

#[test]
fn constant() {
    assert!(WM_KEYUP == 257i32);
    assert!(D3D12_DEFAULT_BLEND_FACTOR_ALPHA == 1f32);
    assert!(UIA_ScrollPatternNoScroll == -1f64);
    assert!(D3DCOMPILER_DLL == "d3dcompiler_47.dll");
}

#[test]
fn function() {
    unsafe {
        let event = CreateEventW(std::ptr::null_mut(), 1, 0, std::ptr::null_mut());
        assert!(event != 0);

        let result = SetEvent(event);
        assert!(result != 0);

        let result = WaitForSingleObject(event, 0);
        assert!(result == 0); // https://github.com/microsoft/win32metadata/issues/77

        let result = CloseHandle(event);
        assert!(result != 0);
    }
}

#[test]
fn interface() -> winrt::Result<()> {
    unsafe {
        let s = winrt::HString::from("https://kennykerr.ca");
        let mut uri = None;

        // TODO: should unwrap with Result<Uri> like WinRT but need https://github.com/microsoft/win32metadata/issues/24
        let hr = CreateUri(s.as_wide().as_ptr() as *mut u16, 1, 0, &mut uri);
        winrt::ErrorCode(hr as u32).ok()?;

        assert!(uri.is_some());

        if let Some(uri) = uri {
            let mut domain = winrt::BString::new();
            let hr = uri.GetDomain(domain.set_abi() as *mut *mut u16);
            winrt::ErrorCode(hr as u32).ok()?;

            assert!(domain == "kennykerr.ca");
        }
    }
    Ok(())
}

#[test]
fn callback() {
    let a: PROPENUMPROCA = callback_a;
    assert!(789 == a(123, "hello a\0".as_ptr() as *const i8, 456));

    let a: PROPENUMPROCW = callback_w;
    assert!(
        789 == a(
            123,
            winrt::HString::from("hello w\0").as_wide().as_ptr(),
            456
        )
    );
}

// TODO: second parameter should be *const i8
extern "system" fn callback_a(param0: isize, param1: *const i8, param2: isize) -> i32 {
    unsafe {
        assert!(param0 == 123);
        assert!(param2 == 456);
        let mut len = 0;
        let mut end = param1;

        loop {
            if *end == 0 {
                break;
            }

            len += 1;
            end = end.add(1);
        }

        let s = String::from_utf8_lossy(std::slice::from_raw_parts(param1 as *const u8, len))
            .into_owned();
        assert!(s == "hello a");
        789
    }
}

// TODO: second parameter should be *const u16
extern "system" fn callback_w(param0: isize, param1: *const u16, param2: isize) -> i32 {
    unsafe {
        assert!(param0 == 123);
        assert!(param2 == 456);
        let mut len = 0;
        let mut end = param1;

        loop {
            if *end == 0 {
                break;
            }

            len += 1;
            end = end.add(1);
        }

        let s = String::from_utf16_lossy(std::slice::from_raw_parts(param1, len));
        assert!(s == "hello w");
        789
    }
}