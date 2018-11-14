extern crate winapi;
extern crate rg;

use winapi::um::d3d11::*;
use winapi::um::d3dcommon::*;
use winapi::um::libloaderapi::*;
use winapi::um::winuser::*;

use winapi::shared::dxgiformat::*;
use winapi::shared::dxgitype::*;
use winapi::shared::dxgi::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;

use winapi::Interface;

use rg::Renderer;

macro_rules! c_str {
    ($str:expr) => (concat!($str, '\0').as_ptr() as *const i8)
}

#[derive(Debug)]
struct RgDx11Renderer {
    texture: *mut ID3D11ShaderResourceView,
    layout: *mut ID3D11InputLayout,
    vs: *mut ID3D11VertexShader,
    ps: *mut ID3D11PixelShader,

    sampler: *mut ID3D11SamplerState,

    camera_buffer: *mut ID3D11Buffer,
    vertex_buffer: *mut ID3D11Buffer,
    index_buffer: *mut ID3D11Buffer,

    blend: *mut ID3D11BlendState,
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
                pSysMem: pixels.as_ptr() as _,
                SysMemPitch: 4,
                SysMemSlicePitch: 0
            };

            (*device).CreateTexture2D(&desc, &data, &mut res as *mut *mut _ as *mut *mut _);

            let mut srv: *mut ID3D11ShaderResourceView = ::std::ptr::null_mut();

            let desc = {
                let mut desc: D3D11_SHADER_RESOURCE_VIEW_DESC = ::std::mem::zeroed();

                desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
                desc.ViewDimension = D3D_SRV_DIMENSION_TEXTURE2D;

                (*desc.u.Texture2D_mut()) = D3D11_TEX2D_SRV {
                    MostDetailedMip: 0,
                    MipLevels: 1
                };

                desc
            };

            (*device).CreateShaderResourceView(res as _, &desc, &mut srv as *mut *mut _ as *mut *mut _);

            srv
        };

        let (layout, vs, ps) = unsafe {
            let mut layout: *mut ID3D11InputLayout = ::std::ptr::null_mut();
            let mut vs: *mut ID3D11VertexShader = ::std::ptr::null_mut();
            let mut ps: *mut ID3D11PixelShader = ::std::ptr::null_mut();

            let vs_bin = include_bytes!("../shaders/vs.o");
            let ps_bin = include_bytes!("../shaders/ps.o");

            (*device).CreateVertexShader(vs_bin.as_ptr() as _, vs_bin.len() as _, ::std::ptr::null_mut(), &mut vs as *mut *mut _ as *mut *mut _);

            let layout_desc = [
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: c_str!("POSITION"),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 0,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: c_str!("TEXCOORD"),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 8,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: c_str!("COLOR"),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    InputSlot: 0,
                    AlignedByteOffset: 16,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0
                },
            ];
            (*device).CreateInputLayout(layout_desc.as_ptr(), layout_desc.len() as _, vs_bin.as_ptr() as _, vs_bin.len() as _, &mut layout as *mut *mut _ as *mut *mut _);
            (*device).CreatePixelShader(ps_bin.as_ptr() as _, ps_bin.len() as _, ::std::ptr::null_mut(), &mut ps as *mut *mut _ as *mut *mut _);

            (layout, vs, ps)
        };

        let sampler = unsafe {
            let mut sampler: *mut ID3D11SamplerState = ::std::ptr::null_mut();

            let desc = D3D11_SAMPLER_DESC {
                Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                AddressU: D3D11_TEXTURE_ADDRESS_WRAP,
                AddressV: D3D11_TEXTURE_ADDRESS_WRAP,
                AddressW: D3D11_TEXTURE_ADDRESS_WRAP,
                MipLODBias: 0f32,
                MaxAnisotropy: 0,
                ComparisonFunc: D3D11_COMPARISON_ALWAYS,
                BorderColor: [0f32, 0f32, 0f32, 0f32],
                MinLOD: 0f32,
                MaxLOD: 0f32
            };

            (*device).CreateSamplerState(&desc, &mut sampler as *mut *mut _ as *mut *mut _);

            sampler
        };

        let (camera_buffer, vertex_buffer, index_buffer) = unsafe {
            let mut camera_buffer: *mut ID3D11Buffer = ::std::ptr::null_mut();
            let mut vertex_buffer: *mut ID3D11Buffer = ::std::ptr::null_mut();
            let mut index_buffer: *mut ID3D11Buffer = ::std::ptr::null_mut();


            let desc = D3D11_BUFFER_DESC {
                ByteWidth: ::std::mem::size_of::<[[f32; 4]; 4]>() as _,
                Usage: D3D11_USAGE_DYNAMIC,
                BindFlags: D3D11_BIND_CONSTANT_BUFFER,
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
                MiscFlags: 0,
                StructureByteStride: 0,
            };
            (*device).CreateBuffer(&desc, ::std::ptr::null_mut(), &mut camera_buffer as *mut *mut _ as *mut *mut _);

            let desc = D3D11_BUFFER_DESC {
                ByteWidth: ::std::mem::size_of::<rg::Vertex>() as u32 * 2000,
                Usage: D3D11_USAGE_DYNAMIC,
                BindFlags: D3D11_BIND_VERTEX_BUFFER,
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
                MiscFlags: 0,
                StructureByteStride: 0,
            };
            (*device).CreateBuffer(&desc, ::std::ptr::null_mut(), &mut vertex_buffer as *mut *mut _ as *mut *mut _);

            let desc = D3D11_BUFFER_DESC {
                ByteWidth: 8192,
                Usage: D3D11_USAGE_DYNAMIC,
                BindFlags: D3D11_BIND_INDEX_BUFFER,
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
                MiscFlags: 0,
                StructureByteStride: 0,
            };
            (*device).CreateBuffer(&desc, ::std::ptr::null_mut(), &mut index_buffer as *mut *mut _ as *mut *mut _);

            (camera_buffer, vertex_buffer, index_buffer)
        };

        let blend = unsafe {
            let mut blend: *mut ID3D11BlendState = ::std::ptr::null_mut();

            let mut desc: D3D11_BLEND_DESC = ::std::mem::zeroed();
            desc.RenderTarget[0] = D3D11_RENDER_TARGET_BLEND_DESC {
                BlendEnable: TRUE,
                SrcBlend: D3D11_BLEND_SRC_ALPHA,
                DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
                BlendOp: D3D11_BLEND_OP_ADD,
                SrcBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
                DestBlendAlpha: D3D11_BLEND_ZERO,
                BlendOpAlpha: D3D11_BLEND_OP_ADD,
                RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL as u8,
            };

            (*device).CreateBlendState(&desc, &mut blend as *mut *mut _ as *mut *mut _);

            blend
        };

        RgDx11Renderer {
            texture,
            layout,
            vs,
            ps,
            sampler,
            camera_buffer,
            vertex_buffer,
            index_buffer,
            blend
        }
    }
}

impl rg::Renderer<*mut ID3D11DeviceContext> for RgDx11Renderer {
    fn render(&mut self, cxt: *mut ID3D11DeviceContext, list: &rg::DrawList) {
        unsafe {
            let mut data: D3D11_MAPPED_SUBRESOURCE = ::std::mem::zeroed();
            let hr = (*cxt).Map(self.camera_buffer as _, 0, D3D11_MAP_WRITE_DISCARD, 0, &mut data as *mut _);
            let ortho = {
                let l = 0f32;
                let r = WIDTH as f32;
                let b = HEIGHT as f32;
                let t = 0f32;

                [
                    2f32 / (r - l),    0f32,              0f32,   0f32,
                    0f32,              2f32 / (t - b),    0f32,   0f32,
                    0f32,              0f32,              0.5f32, 0f32,
                    (r + l) / (l - r), (t + b) / (b - t), 0.5f32, 1f32,
                ]
            };
            ::std::ptr::copy_nonoverlapping::<f32>(ortho.as_ptr() as _, data.pData as _, ortho.len());
            (*cxt).Unmap(self.camera_buffer as _, 0);

            let mut data: D3D11_MAPPED_SUBRESOURCE = ::std::mem::zeroed();
            let hr = (*cxt).Map(self.vertex_buffer as _, 0, D3D11_MAP_WRITE_DISCARD, 0, &mut data as *mut _);
            ::std::ptr::copy_nonoverlapping::<rg::Vertex>(list.vertices.as_ptr(), data.pData as _, list.vertices.len());
            (*cxt).Unmap(self.vertex_buffer as _, 0);

            let mut data: D3D11_MAPPED_SUBRESOURCE = ::std::mem::zeroed();
            let hr = (*cxt).Map(self.index_buffer as _, 0, D3D11_MAP_WRITE_DISCARD, 0, &mut data as *mut _);
            ::std::ptr::copy_nonoverlapping::<u16>(list.indices.as_ptr(), data.pData as _, list.indices.len());
            (*cxt).Unmap(self.index_buffer as _, 0);

            (*cxt).IASetInputLayout(self.layout);

            (*cxt).IASetVertexBuffers(0, 1, &mut self.vertex_buffer as *mut *mut _, &(::std::mem::size_of::<rg::Vertex>() as u32), &0);
            (*cxt).IASetIndexBuffer(self.index_buffer, DXGI_FORMAT_R16_UINT, 0);
            (*cxt).IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

            (*cxt).VSSetShader(self.vs, ::std::ptr::null_mut(), 0);
            (*cxt).VSSetConstantBuffers(0, 1, &self.camera_buffer as *const *mut _);
            (*cxt).PSSetShader(self.ps, ::std::ptr::null_mut(), 0);
            (*cxt).PSSetSamplers(0, 1, &self.sampler as *const *mut _);
            (*cxt).PSSetShaderResources(0, 1, &self.texture as *const *mut _);

            (*cxt).OMSetBlendState(self.blend, &[0f32, 0f32, 0f32, 0f32], 0xffffffff);

            let mut index_offset = 0;
            for command in list.commands() {
                // (*cxt).PSSetShaderResources(0, 1, &command.texture_id as *const *mut _);
                (*cxt).DrawIndexed(command.index_count, index_offset, 0);

                index_offset += command.index_count;
            }
        }
    }
}

const WIDTH: i32 = 1280;
const HEIGHT: i32 = 720;

fn main() {
    let mut list = rg::DrawList::new();


    {
        //let mut path = list.path();

        //path.arc(rg::float2(50f32, 50f32), 20f32, 0f32, 3.14f32, 10).stroke(1f32, false, 0x00ff00ff);

        /*for x in 16..32 {
            let a = 300f32 + (x as f32 + 5f32 * (x as f32) % 5f32).sin() * 220f32;
            let b = 400f32 + ((x/34-5) as f32 + 3f32 * x as f32).cos() * 260f32;
            path = path.line(rg::float2(a, b));
        }

        path.stroke(1f32, false, 0xffff00ff);*/
    }

    list.add_rect_filled(
        rg::float2(400.75f32, 400.25f32),
        rg::float2(500.75f32, 430.25f32),
        //1f32,
        0f32,
        0xff472980
    );

    list.add_text(rg::float2(440f32, 414f32), 0xffffffff, "Test text!");

    unsafe {
        let hwnd = create_window();

        let (swapchain, device, context, backbuffer) = {
            let mut swapchain: *mut IDXGISwapChain = ::std::ptr::null_mut();
            let mut device: *mut ID3D11Device = ::std::ptr::null_mut();
            let mut context: *mut ID3D11DeviceContext = ::std::ptr::null_mut();

            let desc = DXGI_SWAP_CHAIN_DESC {
                BufferDesc: DXGI_MODE_DESC {
                    Width: WIDTH as _,
                    Height: HEIGHT as _,
                    RefreshRate: DXGI_RATIONAL {
                        Denominator: 144,
                        Numerator: 1,
                    },
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
                    Scaling: DXGI_MODE_SCALING_UNSPECIFIED
                },
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 3,
                OutputWindow: hwnd,
                Windowed: TRUE,
                SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
                Flags: 0,
            };

            D3D11CreateDeviceAndSwapChain(
                ::std::ptr::null_mut(),
                D3D_DRIVER_TYPE_HARDWARE,
                ::std::ptr::null_mut(),
                D3D11_CREATE_DEVICE_DEBUG,
                ::std::ptr::null(),
                0,
                D3D11_SDK_VERSION,
                &desc,
                &mut swapchain as *mut *mut _ as *mut *mut _,
                &mut device as *mut *mut _ as *mut *mut _,
                ::std::ptr::null_mut(),
                &mut context as *mut *mut _ as *mut *mut _
            );

            let backbuffer = {
                let mut texture: *mut ID3D11Texture2D = ::std::ptr::null_mut();
                let mut backbuffer: *mut ID3D11RenderTargetView = ::std::ptr::null_mut();

                (*swapchain).GetBuffer(0, &ID3D11Texture2D::uuidof(), &mut texture as *mut *mut _ as *mut *mut _);
                (*device).CreateRenderTargetView(texture as _, ::std::ptr::null(), &mut backbuffer as *mut *mut _ as *mut *mut _);

                backbuffer
            };

            (swapchain, device, context, backbuffer)
        };

        let mut renderer = RgDx11Renderer::new(device);

        println!("{:#?}", renderer);

        let viewport = D3D11_VIEWPORT {
            TopLeftX: 0f32,
            TopLeftY: 0f32,
            Width: WIDTH as f32,
            Height: HEIGHT as f32,
            MinDepth: 0f32,
            MaxDepth: 1f32
        };

        (*context).RSSetViewports(1, &viewport);
        (*context).OMSetRenderTargets(1, &backbuffer, ::std::ptr::null_mut());

        let mut running = true;
        while running {
            let mut msg = ::std::mem::uninitialized();
            while PeekMessageW(&mut msg, ::std::ptr::null_mut(), 0, 0, PM_REMOVE) == TRUE {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);

                if msg.message == WM_QUIT {
                    running = false;
                }
            }

            (*context).ClearRenderTargetView(backbuffer, &[(33f32 / 255f32), (40f32 / 255f32), (45f32 / 255f32), 1f32]);

            renderer.render(context, &list);

            (*swapchain).Present(1, 0);
        }
    }
}


unsafe fn create_window() -> HWND {
    let class_name = register_window_class();

    let title = "rgui".encode_utf16().chain(Some(0)).collect::<Vec<u16>>();

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: WIDTH as _,
        bottom: HEIGHT as _
    };
    AdjustWindowRect(
        &mut rect,
        WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS | WS_VISIBLE,
        FALSE
    );

    CreateWindowExW(
        WS_EX_APPWINDOW | WS_EX_WINDOWEDGE,
        class_name.as_ptr(),
        title.as_ptr() as _,
        WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        rect.right - rect.left,
        rect.bottom - rect.top,
        ::std::ptr::null_mut(),
        ::std::ptr::null_mut(),
        GetModuleHandleW(::std::ptr::null()),
        ::std::ptr::null_mut()
    )
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = "DX11Window".encode_utf16().chain(Some(0)).collect::<Vec<u16>>();

    let class = WNDCLASSEXW {
        cbSize: ::std::mem::size_of::<WNDCLASSEXW>() as _,
        style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
        lpfnWndProc: Some(callback),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: GetModuleHandleW(::std::ptr::null()),
        hIcon: ::std::ptr::null_mut(),
        hCursor: ::std::ptr::null_mut(),
        hbrBackground: ::std::ptr::null_mut(),
        lpszMenuName: ::std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: ::std::ptr::null_mut()
    };

    RegisterClassExW(&class);
    class_name
}

// test comment
//
// TODO: test test test newline
//       more test bla bla.
//
// FIXME: abcedewfwefsdfs
//
unsafe extern "system" fn callback(window: HWND, msg: UINT,
                                   wparam: WPARAM, lparam: LPARAM)
                                   -> LRESULT
{
    // does something
    if msg == WM_DESTROY {
        PostQuitMessage(0);
        return 0;
    }

    DefWindowProcW(window, msg, wparam, lparam)
}
