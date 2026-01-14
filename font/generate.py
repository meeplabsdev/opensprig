from PIL import Image
import json

def get_character(img, col, row):
    cell_w = 4
    cell_h = 6
    left = col * cell_w
    upper = row * cell_h
    right = left + cell_w
    lower = upper + cell_h
    return img.crop((left, upper, right, lower))

def to_hex(img):
    output = []
    for y in range(img.height):
        binary = ""
        for x in range(img.width):
            red = img.getpixel((x, y))[0]
            binary += "1" if red > 0 else "0"
        output.append(hex(int(binary, 2)).ljust(3, "0"))
    return ", ".join(output)

if __name__ == "__main__":
    img = Image.open("ascii.png").convert('RGBA')
    dat = json.load(open("map.json"))

    output = f"#include <cstdint>\n\nauto chars = \""

    for key in dat.keys():
        output += key

    output += f"\";\nuint8_t font[{len(dat.keys())}][6] = {{\n"

    for value in dat.values():
        v = value.split(",")
        character = get_character(img, int(v[0]), int(v[1]))
        output += f"    {{{to_hex(character)}}},\n"

    output += "};"

    with open("font.cpp", "w") as f:
        f.write(output)
