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
    context.register_op("getDimensions", Box::new(op_get_dimensions));
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
fn op_get_dimensions(arg: &[u8], zero_copy: Option<deno::ZeroCopyBuf>) -> deno::CoreOp {
    let filepath = std::str::from_utf8(&arg[..]).unwrap().to_string();
    let fut = async move {
        match image::image_dimensions(filepath) {
            Ok((w, h)) => {
                let mut zc = zero_copy.unwrap();
                let mut_buf = zc.as_mut();
                write_u32(w, mut_buf, 0);
                write_u32(h, mut_buf, 4);
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
fn op_read_image(arg: &[u8], zero_copy: Option<deno::ZeroCopyBuf>) -> deno::CoreOp {
    let filepath = str::from_utf8(&arg[..]).unwrap().to_string();
    let fut = async move {
        match image::open(filepath) {
            Ok(v) => {
                let mut zc = zero_copy.unwrap();
                let img = v.to_rgba();
                let len = (img.width() * img.height()) as usize;
                assert_eq!(zc.len(), len * 4);
                let w = img.width();
                for y in 0..img.height() {
                    for x in 0..w {
                        let p = img.get_pixel(x, y);
                        let image::Rgba(data) = *p;
                        let i = (y * w * 4 + x * 4) as usize;
                        zc[i] = data[0];
                        zc[i + 1] = data[1];
                        zc[i + 2] = data[2];
                        zc[i + 3] = data[3];
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
fn op_save_image(arg: &[u8], zero_copy: Option<deno::ZeroCopyBuf>) -> deno::CoreOp {
    let arg_str = str::from_utf8(&arg[..]).unwrap();
    let req: SaveImageRequest = serde_json::from_str(arg_str).unwrap();
    let fut = async move {
        let zc = zero_copy.unwrap();
        let w = req.width;
        let h = req.height;
        assert_eq!((w * h * 4) as usize, zc.len());
        match image::save_buffer(
            req.filepath, &zc, w, h, image::ColorType::RGBA(8) // 8bit per channel
        ) {
            Ok(_) => Ok(box_ok()),
            Err(_) => Err(()),
        }
    };
    deno::Op::Async(fut.boxed())
}

fn box_ok() -> deno::Buf {
    let result = b"ok";
    Box::new(*result)
}
