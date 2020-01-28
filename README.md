# deno-imaging

Imaging utility for Deno

**!WORK IN PROGRESS!**
## Usage

### 1. Clone source codes

```sh
$ cd your_project
$ git clone https://github.com/keroxp/deno-imaging
```

### 2. Build plugins

Use latest `rustc` and `deno`.

```sh
$ cd deno-imaging
$ cargo build
```

### 3. Import TypeScript functions

```ts
import { readImage, saveImage } from "./deno-imaging/lib.ts";
// Read jpeg image
const image = await readImage("./image.jpg");
// Flip RGB pixeles
for (let y = 0; y < image.height; y++) {
  for (let x = 0; x < image.width; x++) {
    let i = y * image.width * 4 + x * 4;
    let r = image[i];
    let g = image[i + 1];
    let b = image[i + 2];
    image[i] = 255 - r;
    image[i + 1] = 255 - g;
    image[i + 2] = 255 - b;
  }
}
// Save image data as png
await saveImage("./dest.png", image);
```
