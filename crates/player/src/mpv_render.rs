use std::ffi::c_void;
use std::ptr;

use libmpv2_sys as sys;

pub struct RenderContext {
    ctx: *mut sys::mpv_render_context,
}

impl RenderContext {
    pub fn new(mpv: &libmpv2::Mpv) -> libmpv2::Result<Self> {
        let mut params = [
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_API_TYPE,
                data: sys::MPV_RENDER_API_TYPE_SW.as_ptr() as *mut c_void,
            },
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let mut ctx: *mut sys::mpv_render_context = ptr::null_mut();
        let code = unsafe {
            sys::mpv_render_context_create(&mut ctx, mpv.ctx.as_ptr(), params.as_mut_ptr())
        };
        if code < 0 {
            return Err(libmpv2::Error::Raw(code));
        }
        Ok(Self { ctx })
    }
    pub fn update(&self) -> u64 {
        unsafe { sys::mpv_render_context_update(self.ctx) }
    }
    pub fn render(&self, buf: &mut [u8], width: usize, height: usize) -> libmpv2::Result<()> {
        let mut size: [i32; 2] = [width as i32, height as i32];
        let mut stride = width * 4;

        let mut params = [
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_SW_SIZE,
                data: size.as_mut_ptr().cast(),
            },
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_SW_FORMAT,
                data: c"rgb0".as_ptr().cast_mut().cast(),
            },
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_SW_STRIDE,
                data: (&raw mut stride).cast(),
            },
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_SW_POINTER,
                data: buf.as_mut_ptr().cast(),
            },
            sys::mpv_render_param {
                type_: sys::mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let code = unsafe { sys::mpv_render_context_render(self.ctx, params.as_mut_ptr()) };
        if code < 0 {
            return Err(libmpv2::Error::Raw(code));
        }
        Ok(())
    }
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        unsafe { sys::mpv_render_context_free(self.ctx) };
    }
}
