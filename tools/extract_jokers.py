from pathlib import Path
from bs4 import BeautifulSoup
import re


def load_wiki_jokers():
    pages = Path("/root/rulatro/wiki_cache/pages")
    html = (pages / "Jokers.html").read_text()
    soup = BeautifulSoup(html, "html.parser")
    content = soup.find("div", id="mw-content-text")
    table = content.find_all("table")[1]
    rows = table.find_all("tr")
    jokers = []
    for row in rows[1:]:
        cols = row.find_all(["td", "th"])
        if len(cols) < 6:
            continue
        name = cols[1].get_text(" ", strip=True)
        effect = cols[2].get_text(" ", strip=True)
        rarity = cols[4].get_text(" ", strip=True)
        jokers.append((name, effect, rarity))
    return jokers


def load_existing_names():
    text = Path("/root/rulatro/assets/content/jokers.dsl").read_text()
    explicit = set(
        re.findall(r'^joker\s+\S+\s+"([^"]+)"\s+\w+', text, flags=re.M)
    )
    use_names = set()
    for line in text.splitlines():
        line = line.strip()
        if not line.startswith("use "):
            continue
        for match in re.finditer(r'"([^"]+)"', line):
            use_names.add(match.group(1))
    return explicit | use_names


def build_page_index():
    pages = Path("/root/rulatro/wiki_cache/pages")
    files = {p.stem: p for p in pages.glob("*.html")}

    def norm(value: str) -> str:
        return "".join(ch.lower() for ch in value if ch.isalnum())

    norm_map = {norm(stem): path for stem, path in files.items()}
    return files, norm_map, norm


def find_page(name: str, files, norm_map, norm):
    key = name.replace(" ", "_").replace("'", "").replace("’", "")
    return files.get(key) or norm_map.get(norm(key)) or norm_map.get(norm(name.replace(" ", "_")))


def extract_effect(path: Path) -> str:
    soup = BeautifulSoup(path.read_text(), "html.parser")
    content = soup.find("div", id="mw-content-text")
    if not content:
        return "NO_CONTENT"
    text = content.get_text(" ", strip=True)
    match = re.search(r"Effect\\s+(.*?)\\s+Rarity", text)
    if match:
        return match.group(1).strip()
    match = re.search(r"Effect\\s+(.*?)\\s+Type", text)
    if match:
        return match.group(1).strip()
    if "Effect" in text:
        after = text.split("Effect", 1)[1]
        if "Rarity" in after:
            return after.split("Rarity", 1)[0].strip()
        return after[:160].strip()
    return "UNKNOWN"


def main():
    wiki = load_wiki_jokers()
    existing = load_existing_names()
    missing = [(n, e, r) for n, e, r in wiki if n not in existing]

    files, norm_map, norm = build_page_index()

    for name, _, rarity in missing:
        path = find_page(name, files, norm_map, norm)
        if not path:
            print(f"{name} | {rarity} | PAGE_NOT_FOUND")
            continue
        effect = extract_effect(path)
        effect = re.sub(r"\\s+", " ", effect)
        print(f"{name} | {rarity} | {effect}")


if __name__ == "__main__":
    main()
