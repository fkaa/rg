extern crate winapi;
extern crate rg;

use winapi::um::d3d11::*;
use winapi::shared::dxgiformat::*;
use winapi::shared::dxgitype::*;

struct RgDx11Renderer {
    texture: *mut ID3D11ShaderResource
}

impl RgDx11Renderer {
    pub fn new(device: *mut ID3D11Device) -> Self {
        let texture = unsafe {
            let mut res: *mut ID3D11Texture2D = ::std::ptr::null_mut();

            let desc = D3D11_TEXTURE2D_DESC {
                Width: 1,
                Height: 1,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0
                },
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_SHADER_RESOURCE,
                CPUAccessFlags: 0,
                MiscFlags: 0
            };

            let pixels = [0xffu8, 0xffu8, 0xffu8, 0xffu8];
            let data = D3D11_SUBRESOURCE_DATA {
                pSysMem: pixels.as_ptr(),
                SysMemPitch: 4,
                SysMemSlicePitch: 0
            };

            (*device).CreateTexture2D(&desc, &data, &mut res as *mut *mut _ as *mut *mut _);

            let mut srv: *mut ID3D11ShaderResourceView = ::std::ptr::null_mut();

            let desc = {
                let desc: D3D11_SHADER_RESOURCE_VIEW_DESC = ::std::mem::uninitialized();

                desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
                desc.ViewDimension = D3D_SRV_DIMENSION_TEXTURE2D;

                (*desc.u.Texture2D_mut()) = D3D11_TEX2D_SRV {
                    MostDetailedMip: 0,
                    MipLevels: 1
                };

                desc
            };

            (*device).CreateShaderResourceView(res, &desc, &mut src as *mut *mut _ as *mut *mut _);

            srv
        };

        RgDx11Renderer {
            texture
        }
    }
}

impl rg::Renderer for RgDx11Renderer {
    fn render(list: &rg::DrawList) {
        for command in list.commands() {

        }
    }
}

fn main() {
    let renderer = RgDx11Renderer::new();
    let mut list = rg::DrawList::new();

    list.path()
        .line(rg::float2(100f32, 100f32))
        .line(rg::float2(150f32, 150f32))
        .line(rg::float2(200f32, 100f32))
        .line(rg::float2(250f32, 150f32))
        .stroke(0xff0000ff);


}
