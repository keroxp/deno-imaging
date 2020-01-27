import * as path from "./vendor/https/deno.land/std/path/mod.ts";
const filenameBase = "deno_imaging";
let filenameSuffix = ".so";
let filenamePrefix = "lib";

if (Deno.build.os === "win") {
  filenameSuffix = ".dll";
  filenamePrefix = "";
}
if (Deno.build.os === "mac") {
  filenameSuffix = ".dylib";
}

const filename = `./target/debug/${filenamePrefix}${filenameBase}${filenameSuffix}`;
const pluginPath = new URL(filename, import.meta.url).pathname;
const plugin = Deno.openPlugin(pluginPath);

export interface Size {
  width: number,
  height: number
}
export type RawImageData = Uint8Array & Size;

const encoder = new TextEncoder;
export async function getMetadata(path: string): Promise<Size> {
  const arg = encoder.encode(path);
  const buf = new ArrayBuffer(8);
  const dest = new DataView(buf);
  const func = plugin.ops.getMetadata;
  return new Promise<Size>(resolve => {
    func.setAsyncHandler(resp => {
      const width = dest.getUint32(0);
      const height = dest.getUint32(4);
      resolve({width, height});
    });
    func.dispatch(arg, dest);
  });
}

export async function readImage(path: string): Promise<RawImageData> {
  const arg = encoder.encode(path);
  const size = await getMetadata(path);
  const buf = new Uint8Array(size.width*size.height*4);
  const func = plugin.ops.readImage;
  return new Promise<RawImageData>(resolve => {
    func.setAsyncHandler(() => {
      resolve(Object.assign(buf, size));
    });
    func.dispatch(arg, buf);
  });
}

type SaveImageRequest = {
  filepath: string
  width: number
  height: number
}
export async function saveImage(filepath: string, image: RawImageData): Promise<void>{
  const dir = path.dirname(filepath);
  await Deno.mkdir(dir, {recursive: true});
  const req: SaveImageRequest = {
    filepath,
    width: image.width,
    height: image.height
  };
  const func = plugin.ops.saveImage;
  return new Promise(resolve => {
    func.setAsyncHandler(() => {
      resolve()
    });
    func.dispatch(encoder.encode(JSON.stringify(req)), image);
  });
}