#!/usr/bin/env python3
"""Compte les lignes physiques Rust, par crate, sans dépendance externe.

Catégories exclusives : code hors tests, commentaires seuls, code de tests,
lignes vides. Une ligne avec du code et un commentaire compte comme du code.
Les tests incluent les fichiers tests.rs, les dossiers tests/, les éléments
#[cfg(test)] et les fonctions #[test]. Les cfg composés ne sont pas analysés.
"""

import argparse
import bisect
import re
from collections import Counter, defaultdict
from pathlib import Path


def scan(source):
    """Masque commentaires et littéraux, en conservant positions et lignes."""
    masked = list(source)
    comment_lines = set()
    line = 0
    i = 0
    while i < len(source):
        start = i
        is_comment = False
        if source.startswith("//", i):
            end = source.find("\n", i)
            i = len(source) if end == -1 else end
            is_comment = True
        elif source.startswith("/*", i):
            depth = 1
            i += 2
            while i < len(source) and depth:
                if source.startswith("/*", i):
                    depth += 1
                    i += 2
                elif source.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
            is_comment = True
        else:
            raw = re.match(r'(?:br|cr|r)(#*)"', source[i:]) if source[i] in "bcr" else None
            if raw:
                delimiter = '"' + raw.group(1)
                end = source.find(delimiter, i + raw.end())
                i = len(source) if end == -1 else end + len(delimiter)
            elif source[i] == '"':
                i += 1
                while i < len(source):
                    if source[i] == "\\":
                        i += 2
                    elif source[i] == '"':
                        i += 1
                        break
                    else:
                        i += 1
                i = min(i, len(source))
            elif source[i] == "'" and (char := re.match(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|[^\n])|[^'\\\n])'", source[i:])):
                i += char.end()
            else:
                if source[i] == "\n":
                    line += 1
                i += 1
                continue
        for pos in range(start, i):
            if is_comment:
                comment_lines.add(line)
            if source[pos] == "\n":
                line += 1
            else:
                masked[pos] = " " if is_comment else "x"
    return "".join(masked), comment_lines


def test_lines(masked):
    """Repère les éléments de test usuels, y compris leurs attributs."""
    line_starts = [0] + [m.end() for m in re.finditer("\n", masked)]
    lines = set()
    attributes = re.compile(r"#\s*\[\s*(?:cfg\s*\(\s*test\s*\)|test)\s*\]")
    for match in attributes.finditer(masked):
        opening = re.search(r"[;{]", masked[match.end():])
        if not opening:
            continue
        end = match.end() + opening.end()
        if opening.group() == "{":
            depth = 1
            while end < len(masked) and depth:
                if masked[end] == "{":
                    depth += 1
                elif masked[end] == "}":
                    depth -= 1
                end += 1
        first = bisect.bisect_right(line_starts, match.start()) - 1
        last = bisect.bisect_right(line_starts, end - 1) - 1
        lines.update(range(first, last + 1))
    return lines


def count_file(path, root):
    source = path.read_text(encoding="utf-8")
    masked, comments = scan(source)
    relative = path.relative_to(root)
    whole_test = path.name == "tests.rs" or "tests" in relative.parts[:-1]
    tests = test_lines(masked) if not whole_test else set()
    counts = Counter()
    for idx, content in enumerate(masked.splitlines()):
        if content.strip():
            counts["tests" if whole_test or idx in tests else "code"] += 1
        elif idx in comments:
            counts["comments"] += 1
        else:
            counts["blank"] += 1
    return counts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", nargs="?", type=Path, default=Path(__file__).resolve().parent.parent,
                        help="Racine à analyser (par défaut : racine du projet)")
    parser.add_argument("--files", action="store_true", help="Détail par fichier plutôt que par crate")
    args = parser.parse_args()
    root = args.root.resolve()
    if not root.is_dir():
        parser.error(f"Dossier introuvable : {root}")
    groups = defaultdict(Counter)
    for path in sorted(root.rglob("*.rs")):
        relative = path.relative_to(root)
        if any(part == "target" or part.startswith(".") for part in relative.parts):
            continue
        group = str(relative) if args.files else (
            relative.parts[1] if relative.parts[0] == "crates" and len(relative.parts) > 2 else "autres"
        )
        groups[group].update(count_file(path, root))
    headers = ("Code", "Commentaires", "Tests", "Vides", "Total")
    width = max([len("TOTAL"), *(len(name) for name in groups)])
    print(f"{'Fichier' if args.files else 'Crate':<{width}} " + " ".join(f"{h:>12}" for h in headers))
    total = Counter()
    for name, counts in groups.items():
        print_row(name, counts, width)
        total.update(counts)
    print_row("TOTAL", total, width)
    print("\nLignes physiques .rs ; commentaires seuls séparés, lignes mixtes comptées comme code/tests.")


def print_row(name, counts, width):
    values = [counts[key] for key in ("code", "comments", "tests", "blank")]
    print(f"{name:<{width}} " + " ".join(f"{n:>12}" for n in [*values, sum(values)]))


if __name__ == "__main__":
    main()
