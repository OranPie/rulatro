#!/usr/bin/env python3
import json
import re
from pathlib import Path

from bs4 import BeautifulSoup

ROOT = Path(__file__).resolve().parents[1]
WIKI_DIR = ROOT / "wiki_cache" / "pages"


def read_html(slug: str) -> str | None:
    path = WIKI_DIR / f"{slug}.html"
    if not path.exists():
        return None
    return path.read_text(encoding="utf-8", errors="replace")


def extract_listing_slugs(html: str) -> list[str]:
    soup = BeautifulSoup(html, "html.parser")
    content = soup.find(id="mw-content-text") or soup
    slugs: set[str] = set()

    for table in content.find_all("table"):
        for row in table.find_all("tr"):
            link = row.find("a", href=True)
            if not link:
                continue
            href = link.get("href", "")
            if not href.startswith("/w/"):
                continue
            if ":" in href or "#" in href:
                continue
            slug = href.split("/w/", 1)[1]
            if slug:
                slugs.add(slug)

    for gallery in content.find_all("div", class_=lambda c: c and "gallery" in c):
        for link in gallery.find_all("a", href=True):
            href = link.get("href", "")
            if not href.startswith("/w/"):
                continue
            if ":" in href or "#" in href:
                continue
            slug = href.split("/w/", 1)[1]
            if slug:
                slugs.add(slug)

    return sorted(slugs)


def extract_infobox_text(slug: str, header: str) -> str:
    html = read_html(slug)
    if not html:
        return ""
    soup = BeautifulSoup(html, "html.parser")
    for group in soup.select(".infobox-group"):
        head = group.find(class_="infobox-header")
        if not head:
            continue
        if head.get_text(strip=True).lower() == header.lower():
            text = group.get_text(" ", strip=True)
            return text.replace(head.get_text(strip=True), "", 1).strip()
    return ""


def slug_from_name(name: str) -> str:
    return name.strip().replace(" ", "_")


def load_json(path: Path) -> list[dict]:
    return json.loads(path.read_text(encoding="utf-8"))


def summarize_effects(effects: list[dict]) -> str:
    parts: list[str] = []
    for block in effects:
        for eff in block.get("effects", []):
            if isinstance(eff, str):
                parts.append(eff)
                continue
            if isinstance(eff, dict):
                for key, val in eff.items():
                    if isinstance(val, dict):
                        if key == "EnhanceSelected":
                            parts.append(
                                f"Enhance({val.get('enhancement')} x{val.get('count')})"
                            )
                        elif key == "AddEditionToSelected":
                            parts.append(
                                f"AddEdition({','.join(val.get('editions', []))} x{val.get('count')})"
                            )
                        elif key == "AddSealToSelected":
                            parts.append(f"AddSeal({val.get('seal')} x{val.get('count')})")
                        elif key == "ConvertSelectedSuit":
                            parts.append(
                                f"ConvertSuit({val.get('suit')} x{val.get('count')})"
                            )
                        elif key == "IncreaseSelectedRank":
                            parts.append(
                                f"IncreaseRank(x{val.get('count')} {val.get('delta'):+})"
                            )
                        elif key == "DestroySelected":
                            parts.append(f"DestroySelected(x{val.get('count')})")
                        elif key == "DestroyRandomInHand":
                            parts.append(f"DestroyRandom(x{val.get('count')})")
                        elif key == "CopySelected":
                            parts.append(f"CopySelected(x{val.get('count')})")
                        elif key == "AddRandomConsumable":
                            parts.append(
                                f"AddConsumable({val.get('kind')} x{val.get('count')})"
                            )
                        elif key == "AddJoker":
                            parts.append(
                                f"AddJoker({val.get('rarity')} x{val.get('count')})"
                            )
                        elif key == "AddRandomJoker":
                            parts.append(f"AddRandomJoker(x{val.get('count')})")
                        elif key == "UpgradeHand":
                            parts.append(
                                f"UpgradeHand({val.get('hand')} x{val.get('amount')})"
                            )
                        elif key == "UpgradeAllHands":
                            parts.append(f"UpgradeAllHands(x{val.get('amount')})")
                        elif key == "RandomJokerEdition":
                            parts.append(
                                f"RandomJokerEdition({','.join(val.get('editions', []))} {val.get('chance')})"
                            )
                        elif key == "SetRandomJokerEdition":
                            parts.append(f"SetRandomJokerEdition({val.get('edition')})")
                        elif key == "SetRandomJokerEditionDestroyOthers":
                            parts.append(
                                f"SetJokerEditionDestroyOthers({val.get('edition')})"
                            )
                        elif key == "DuplicateRandomJokerDestroyOthers":
                            parts.append(
                                f"DuplicateJokerDestroyOthers(remove_negative={val.get('remove_negative')})"
                            )
                        elif key == "AddRandomEnhancedCards":
                            parts.append(
                                f"AddEnhancedCards({val.get('filter')} x{val.get('count')})"
                            )
                        elif key == "CreateLastConsumable":
                            parts.append(
                                f"CreateLastConsumable(exclude={val.get('exclude')})"
                            )
                        else:
                            parts.append(f"{key}({val})")
                    else:
                        parts.append(f"{key}({val})")
    return "; ".join(parts)


def parse_tags_dsl(path: Path) -> list[dict]:
    tags = []
    current = None
    brace_depth = 0
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line.startswith("#") or not line:
            continue
        if line.startswith("tag "):
            match = re.match(r'tag\s+(\S+)\s+"([^"]+)"\s*\{', line)
            if match:
                current = {"id": match.group(1), "name": match.group(2), "body": []}
                brace_depth = 1
            continue
        if current is not None:
            brace_depth += line.count("{")
            brace_depth -= line.count("}")
            if brace_depth <= 0:
                tags.append(current)
                current = None
                continue
            if line and line != "}":
                current["body"].append(line)
    return tags


def report_category(title: str, wiki_slugs: list[str], local_items: list[dict], effect_header: str):
    local_by_slug = {slug_from_name(item["name"]): item for item in local_items}
    wiki_set = set(wiki_slugs)
    local_set = set(local_by_slug.keys())

    missing = sorted(wiki_set - local_set)
    extra = sorted(local_set - wiki_set)

    print(f"\n== {title} ==")
    print(f"wiki: {len(wiki_slugs)} | local: {len(local_items)}")
    if missing:
        print("missing (wiki -> local):", ", ".join(missing))
    if extra:
        print("extra (local -> wiki):", ", ".join(extra))

    for slug in wiki_slugs:
        local = local_by_slug.get(slug)
        if not local:
            continue
        wiki_text = extract_infobox_text(slug, effect_header)
        local_effects = summarize_effects(local.get("effects", []))
        print(f"- {local['name']} | wiki: {wiki_text} | local: {local_effects}")


def report_tags(wiki_slugs: list[str], local_items: list[dict]):
    local_by_slug = {slug_from_name(item["name"]): item for item in local_items}
    wiki_set = set(wiki_slugs)
    local_set = set(local_by_slug.keys())

    missing = sorted(wiki_set - local_set)
    extra = sorted(local_set - wiki_set)

    print("\n== Tags ==")
    print(f"wiki: {len(wiki_slugs)} | local: {len(local_items)}")
    if missing:
        print("missing (wiki -> local):", ", ".join(missing))
    if extra:
        print("extra (local -> wiki):", ", ".join(extra))

    for slug in wiki_slugs:
        local = local_by_slug.get(slug)
        if not local:
            continue
        wiki_text = extract_infobox_text(slug, "description")
        body = " ".join(local.get("body", []))
        print(f"- {local['name']} | wiki: {wiki_text} | local: {body}")


def main() -> None:
    tarot_list = read_html("Tarot_Cards")
    planet_list = read_html("Planet_Cards")
    spectral_list = read_html("Spectral_Cards")
    tag_list = read_html("Tags")
    if not all([tarot_list, planet_list, spectral_list, tag_list]):
        print("missing wiki cache pages; run tools/wiki_fetch.py first")
        return

    tarots = load_json(ROOT / "assets" / "content" / "tarots.json")
    planets = load_json(ROOT / "assets" / "content" / "planets.json")
    spectrals = load_json(ROOT / "assets" / "content" / "spectrals.json")
    tags = parse_tags_dsl(ROOT / "assets" / "content" / "tags.dsl")

    report_category(
        "Tarot Cards",
        extract_listing_slugs(tarot_list),
        tarots,
        "effect",
    )
    report_category(
        "Planet Cards",
        extract_listing_slugs(planet_list),
        planets,
        "effect",
    )
    report_category(
        "Spectral Cards",
        extract_listing_slugs(spectral_list),
        spectrals,
        "effect",
    )
    report_tags(extract_listing_slugs(tag_list), tags)


if __name__ == "__main__":
    main()
