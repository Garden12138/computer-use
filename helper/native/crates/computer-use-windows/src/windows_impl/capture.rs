use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use computer_use_core::{write_png_rgb, HelperError};
use serde_json::{json, Value};
use windows::core::Interface;
use windows::Graphics::Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D,
    D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAPPED_SUBRESOURCE,
    D3D11_MAP_READ, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::Gdi::{MonitorFromWindow, HMONITOR, MONITOR_DEFAULTTOPRIMARY};
use windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11DeviceFromDXGIDevice;
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::Win32::UI::WindowsAndMessaging::{GetDesktopWindow, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

use super::dpi;

pub fn screen_size(scale: f64) -> Value {
    let (w, h) = dpi::screen_pixels();
    json!({
        "width": w as f64 * scale,
        "height": h as f64 * scale,
        "width_points": w,
        "height_points": h,
        "scale": scale,
    })
}

pub fn screenshot(path: Option<&str>, hwnd: Option<HWND>, scale: f64) -> Result<Value, HelperError> {
    let rgb = capture_rgb(hwnd)?;
    let out = output_path(path);
    write_png_rgb(&out, rgb.0, rgb.1, &rgb.2)?;
    let (sw, sh) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    let mut payload = json!({
        "path": out.to_string_lossy(),
        "width": rgb.0,
        "height": rgb.1,
        "scale": scale,
        "display_width_points": sw,
        "display_height_points": sh,
    });
    if let Some(h) = hwnd {
        payload["window_id"] = json!(h.0 as u64);
    }
    Ok(payload)
}

fn output_path(path: Option<&str>) -> PathBuf {
    if let Some(p) = path {
        return PathBuf::from(p);
    }
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    std::env::temp_dir().join(format!("computer-use-{ms}.png"))
}

fn capture_rgb(hwnd: Option<HWND>) -> Result<(u32, u32, Vec<u8>), HelperError> {
    unsafe { capture_rgb_inner(hwnd) }
}

unsafe fn capture_rgb_inner(hwnd: Option<HWND>) -> Result<(u32, u32, Vec<u8>), HelperError> {
    let mut device: Option<ID3D11Device> = None;
    D3D11CreateDevice(
        None,
        D3D_DRIVER_TYPE_HARDWARE,
        None,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        Some(&[D3D_FEATURE_LEVEL_11_0]),
        D3D11_SDK_VERSION,
        Some(&mut device),
        None,
        None,
    )
    .map_err(|e| HelperError::failed(e.to_string()))?;
    let d3d = device.ok_or_else(|| HelperError::failed("D3D11 device"))?;
    let dxgi: IDXGIDevice = d3d.cast().map_err(|e| HelperError::failed(e.to_string()))?;
    let inspectable = CreateDirect3D11DeviceFromDXGIDevice(&dxgi).map_err(|e| HelperError::failed(e.to_string()))?;
    let winrt_device: IDirect3DDevice = inspectable.cast().map_err(|e| HelperError::failed(e.to_string()))?;

    let interop: IGraphicsCaptureItemInterop =
        windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
            .map_err(|e| HelperError::failed(e.to_string()))?;
    let item: GraphicsCaptureItem = if let Some(hwnd) = hwnd {
        interop.CreateForWindow(hwnd).map_err(|e| HelperError::failed(e.to_string()))?
    } else {
        let monitor: HMONITOR = MonitorFromWindow(GetDesktopWindow(), MONITOR_DEFAULTTOPRIMARY);
        interop.CreateForMonitor(monitor).map_err(|e| HelperError::failed(e.to_string()))?
    };

    let size = item.Size().map_err(|e| HelperError::failed(e.to_string()))?;
    let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
        &winrt_device,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        1,
        size,
    )
    .map_err(|e| HelperError::failed(e.to_string()))?;
    let session: GraphicsCaptureSession = pool.CreateCaptureSession(&item).map_err(|e| HelperError::failed(e.to_string()))?;
    session.StartCapture().map_err(|e| HelperError::failed(e.to_string()))?;

    let frame = {
        let mut got = None;
        for _ in 0..50 {
            if let Ok(f) = pool.TryGetNextFrame() {
                got = Some(f);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        got.ok_or_else(|| HelperError::failed("no Graphics Capture frame"))?
    };

    let surface = frame.Surface().map_err(|e| HelperError::failed(e.to_string()))?;
    let access: windows::Win32::System::WinRT::Direct3D11::IDirect3DDxgiInterfaceAccess =
        surface.cast().map_err(|e| HelperError::failed(e.to_string()))?;
    let texture: ID3D11Texture2D = access.GetInterface().map_err(|e| HelperError::failed(e.to_string()))?;

    let mut desc = D3D11_TEXTURE2D_DESC::default();
    texture.GetDesc(&mut desc);
    desc.Usage = D3D11_USAGE_STAGING;
    desc.BindFlags = 0;
    desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
    desc.MiscFlags = 0;
    desc.SampleDesc = DXGI_SAMPLE_DESC { Count: 1, Quality: 0 };
    desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;

    let mut staging: Option<ID3D11Texture2D> = None;
    d3d.CreateTexture2D(&desc, None, Some(&mut staging))
        .map_err(|e| HelperError::failed(e.to_string()))?;
    let staging = staging.ok_or_else(|| HelperError::failed("staging texture"))?;
    let ctx: ID3D11DeviceContext = d3d.GetImmediateContext().map_err(|e| HelperError::failed(e.to_string()))?;
    let src: ID3D11Resource = texture.cast().map_err(|e| HelperError::failed(e.to_string()))?;
    let dst: ID3D11Resource = staging.cast().map_err(|e| HelperError::failed(e.to_string()))?;
    ctx.CopyResource(&dst, &src);

    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    ctx.Map(&dst, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
        .map_err(|e| HelperError::failed(e.to_string()))?;
    let width = desc.Width;
    let height = desc.Height;
    let pitch = mapped.RowPitch as usize;
    let src_ptr = mapped.pData as *const u8;
    let mut rgb = vec![0u8; width as usize * height as usize * 3];
    for y in 0..height as usize {
        let row = std::slice::from_raw_parts(src_ptr.add(y * pitch), width as usize * 4);
        for x in 0..width as usize {
            let i = x * 4;
            let o = (y * width as usize + x) * 3;
            rgb[o] = row[i + 2];
            rgb[o + 1] = row[i + 1];
            rgb[o + 2] = row[i];
        }
    }
    ctx.Unmap(&dst, 0);
    let _ = session.Close();
    let _ = pool.Close();
    Ok((width, height, rgb))
}

#[allow(dead_code)]
fn _path(_: &Path) {}
