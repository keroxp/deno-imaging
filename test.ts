import { test, runIfMain } from "./vendor/https/deno.land/std/testing/mod.ts";
import { assertEquals } from "./vendor/https/deno.land/std/testing/asserts.ts";
import { getMetadata, readImage, saveImage } from "./src/lib.ts";

test("getMetadata1", async () => {
  const size = await getMetadata("./fixtures/deno_150x100.jpg");
  assertEquals(size.width, 150);
  assertEquals(size.height, 100);
});
test("read", async () => {
  const image = await readImage("./fixtures/black_150x100.jpg");
  for (let y = 0; y < image.height; y++) {
    for (let x = 0; x < image.width; x++) {
      const i = y * x * 4 + x * 4;
      assertEquals(image[i], 0);
      assertEquals(image[i + 1], 0);
      assertEquals(image[i + 2], 0);
      assertEquals(image[i + 3], 255);
    }
  }
});
test("read/flip/save", async () => {
  const image = await readImage("./fixtures/deno_150x100.jpg");
  assertEquals(image.width, 150);
  assertEquals(image.height, 100);
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
  await saveImage("./tmp/dest.png", image);
});
runIfMain(import.meta);
