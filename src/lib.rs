extern crate image;
#[macro_use]
extern crate deno_core as deno;
extern crate futures;
extern crate serde;
extern crate serde_json;

use deno::PluginInitContext;
use futures::future::FutureExt;
use serde::Deserialize;
use std::str;

fn init(context: &mut dyn PluginInitContext) {
    context.register_op("getMetadata", Box::new(op_get_metadata));
    context.register_op("readImage", Box::new(op_read_image));
    context.register_op("saveImage", Box::new(op_save_image));
}
init_fn!(init);

fn write_u32(v: u32, dest: &mut [u8], offs: usize) {
    let b = v.to_be_bytes();
    for i in 0..4 {
        dest[offs + i] = b[i]
    }
}

/**
 * arg: string (filepath)
 * zero_copy: [u8;8]
 * - width: u32
 * - height: u32
 */
fn op_get_metadata(arg: &[u8], zero_copy: Option<deno::PinnedBuf>) -> deno::CoreOp {
    let filepath = std::str::from_utf8(&arg[..]).unwrap().to_string();
    let fut = async move {
        match image::open(filepath) {
            Ok(v) => {
                let img = v.to_rgba();
                let mut zc = zero_copy.unwrap();
                let mut_buf = zc.as_mut();
                write_u32(img.width(), mut_buf, 0);
                write_u32(img.height(), mut_buf, 4);
                Ok(box_ok())
            }
            Err(_) => Err(()),
        }
    };
    deno::Op::Async(fut.boxed())
}

/**
 * arg: string (filepath)
 * zero_copy: [u8:n]
 * - dest for RGBA image
 */
fn op_read_image(arg: &[u8], zero_copy: Option<deno::PinnedBuf>) -> deno::CoreOp {
    let filepath = str::from_utf8(&arg[..]).unwrap().to_string();
    let fut = async move {
        match image::open(filepath) {
            Ok(v) => {
                let mut zc = zero_copy.unwrap();
                let buf = zc.as_mut();
                let img = v.to_rgba();
                let len = (img.width() * img.height()) as usize;
                assert_eq!(buf.len(), len * 4);
                let w = img.width();
                for y in 0..img.height() {
                    for x in 0..w {
                        let p = img.get_pixel(x, y);
                        let image::Rgba(data) = *p;
                        let i = (y * w * 4 + x * 4) as usize;
                        buf[i] = data[0];
                        buf[i + 1] = data[1];
                        buf[i + 2] = data[2];
                        buf[i + 3] = data[3];
                    }
                }
                Ok(box_ok())
            }
            Err(_) => Err(()),
        }
    };
    deno::Op::Async(fut.boxed())
}

#[derive(Deserialize)]
struct SaveImageRequest {
    filepath: String,
    width: u32,
    height: u32,
}

/**
 * arg: string (SaveImageRequest)
 * zero_copy: [u8] (Raw rgba image buffer)
 */
fn op_save_image(arg: &[u8], zero_copy: Option<deno::PinnedBuf>) -> deno::CoreOp {
    let arg_str = str::from_utf8(&arg[..]).unwrap();
    let req: SaveImageRequest = serde_json::from_str(arg_str).unwrap();
    let fut = async move {
        let zc = zero_copy.unwrap();
        let mut img = image::ImageBuffer::new(req.width, req.height);
        let w = req.width;
        assert_eq!((req.width * req.height * 4) as usize, zc.len());
        for y in 0..req.height {
            for x in 0..w {
                let i = (y * w * 4 + x * 4) as usize;
                let r = zc[i];
                let g = zc[i + 1];
                let b = zc[i + 2];
                let a = zc[i + 3];
                let pixel = img.get_pixel_mut(x, y);
                *pixel = image::Rgba([r, g, b, a]);
            }
        }
        match img.save(req.filepath) {
            Ok(_) => Ok(box_ok()),
            Err(_) => Err(()),
        }
    };
    deno::Op::Async(fut.boxed())
}

fn box_ok() -> deno::Buf {
    // Boxed result size must be a multiple of 4... :(
    let result = b"opok";
    Box::new(*result)
}
