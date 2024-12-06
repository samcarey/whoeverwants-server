import os
import sys


def read_file(file_path):
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            return f.read()
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
        return None


def read_files_from_folder(folder_path):
    file_contents = []
    for root, _, files in os.walk(folder_path):
        for file in files:
            file_path = os.path.join(root, file)
            content = read_file(file_path)
            if content is not None:
                file_contents.append((file_path, content))
    return file_contents


def main(output_file, *sources):
    all_contents = []
    for source in sources:
        if os.path.isdir(source):
            all_contents.extend(read_files_from_folder(source))
        elif os.path.isfile(source):
            content = read_file(source)
            if content is not None:
                all_contents.append((source, content))
        else:
            print(f"{source} is not a valid file or directory.")

    with open(output_file, "w", encoding="utf-8") as f:
        for file_path, content in all_contents:
            f.write(f"# File: {file_path}\n")
            f.write("```\n")
            f.write(content)
            f.write("\n```\n\n")
            f.write(header)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python gather_context.py source1 [source2 ... sourceN]")
    else:
        output_file = "context.md"
        sources = sys.argv[2:]
        main(output_file, *sources)


header = """
Instructions: If you tell me a function needs to change, output the whole function, not just new pieces interleaved with old.
Do not try to hold a mutex lock across an await point.
"""
