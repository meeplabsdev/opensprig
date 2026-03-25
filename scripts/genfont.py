from PIL import Image
img = Image.open("font4x6.png")
img = img.convert("RGB")
px = img.load()

with open("out", "w") as f:
    f.write("// ASCII 32 -> 127\n")
    f.write("pub const FOURBYSIX: [u32; 96] = [\n")

    for y in range(0, 36, 6):
        for x in range(0, 64, 4):
            f.write("    0b")
            for dy in range(y, y + 6):
                for dx in range(x, x + 4):
                    f.write("1" if px[dx, dy][0] == 255 else "0")

            f.write(",\n")
    f.write("];\n")