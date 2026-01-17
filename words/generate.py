if __name__ == "__main__":
    words = []

    with open("words.txt", "r") as f:
        words = [word.strip() for word in f.readlines()]

    with open("words.cpp", "w") as f:
        f.write(f"#define WORDS_COUNT {len(words)}\n")
        f.write("const char *words[WORDS_COUNT] = {\n\t\"" + "\",\n\t\"".join(words) + "\",\n};\n")

