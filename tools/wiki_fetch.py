#!/usr/bin/env python3
import json
import os
import re
import time
from dataclasses import dataclass
from html.parser import HTMLParser
from urllib.parse import urljoin, urlparse, urldefrag, unquote
from urllib.request import Request, urlopen

BASE = "https://balatrowiki.org"
CACHE_DIR = os.path.join(os.path.dirname(__file__), "..", "wiki_cache")
PAGES_DIR = os.path.join(CACHE_DIR, "pages")

SEEDS = [
    "/w/Jokers",
    "/w/Hands",
    "/w/Scoring",
    "/w/Blind",
    "/w/Blinds",
    "/w/Ante",
    "/w/Shop",
    "/w/Booster_Pack",
    "/w/Booster_Packs",
    "/w/Tarot_Cards",
    "/w/Planet_Cards",
    "/w/Spectral_Cards",
    "/w/Tags",
    "/w/Tag",
    "/w/Tarot",
    "/w/Planet",
    "/w/Spectral",
]

RULE_SLUGS = {
    "Hands",
    "Scoring",
    "Blind",
    "Blinds",
    "Ante",
    "Shop",
    "Booster_Pack",
    "Booster_Packs",
    "Tarot_Cards",
    "Planet_Cards",
    "Spectral_Cards",
    "Tags",
    "Tag",
    "Tarot",
    "Planet",
    "Spectral",
}

LISTING_SLUGS = {
    "Jokers",
    "Tarot_Cards",
    "Planet_Cards",
    "Spectral_Cards",
    "Tags",
    "Tag",
}

USER_AGENT = "rulatro-bot/0.2 (local cache for rules research)"
SLEEP_SECONDS = 0.6


class LinkParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []

    def handle_starttag(self, tag, attrs):
        if tag != "a":
            return
        for key, value in attrs:
            if key == "href" and value:
                self.links.append(value)


@dataclass
class Page:
    url: str
    slug: str
    path: str


def fetch(url: str) -> str:
    req = Request(url, headers={"User-Agent": USER_AGENT})
    with urlopen(req, timeout=20) as resp:
        return resp.read().decode("utf-8", errors="replace")


def normalize_url(url: str) -> str | None:
    url, _ = urldefrag(url)
    if not url.startswith("http"):
        url = urljoin(BASE, url)
    parsed = urlparse(url)
    if parsed.netloc != urlparse(BASE).netloc:
        return None
    if not (parsed.path.startswith("/wiki/") or parsed.path.startswith("/w/")):
        return None
    if ":" in parsed.path:
        return None
    if parsed.query:
        url = url.split("?", 1)[0]
    return url


def slug_from_path(path: str) -> str:
    if path.startswith("/wiki/"):
        slug = path.split("/wiki/", 1)[-1]
    else:
        slug = path.split("/w/", 1)[-1]
    slug = unquote(slug)
    return slug


def is_joker_slug(slug: str, joker_slugs: set[str]) -> bool:
    return slug in joker_slugs or "joker" in slug.lower()


def extract_listing_slugs(html: str) -> set[str]:
    try:
        from bs4 import BeautifulSoup  # type: ignore
    except Exception:
        return set()

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

    return slugs


def allowed_slug(slug: str, joker_slugs: set[str], extra_slugs: set[str]) -> bool:
    return (
        is_joker_slug(slug, joker_slugs)
        or slug in extra_slugs
        or slug in RULE_SLUGS
        or slug in LISTING_SLUGS
    )


def save_page(page: Page, html: str) -> None:
    os.makedirs(PAGES_DIR, exist_ok=True)
    safe = re.sub(r"[^a-zA-Z0-9_.-]+", "_", page.slug)
    filename = f"{safe}.html"
    path = os.path.join(PAGES_DIR, filename)
    with open(path, "w", encoding="utf-8") as f:
        f.write(html)
    page.path = path


def extract_joker_slugs(html: str) -> set[str]:
    try:
        from bs4 import BeautifulSoup  # type: ignore
    except Exception:
        return set()

    soup = BeautifulSoup(html, "html.parser")
    anchor = soup.find("a", href="/w/Greedy_Joker")
    if not anchor:
        return set()
    table = anchor.find_parent("table")
    if not table:
        return set()
    slugs: set[str] = set()
    for row in table.find_all("tr"):
        cells = row.find_all("td")
        if not cells:
            continue
        cell = cells[1] if len(cells) > 1 else cells[0]
        link = cell.find("a", href=True)
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
    return slugs


def main() -> None:
    os.makedirs(CACHE_DIR, exist_ok=True)
    queue = []
    visited_urls: set[str] = set()
    visited_slugs: set[str] = set()
    pages: list[Page] = []
    joker_slugs: set[str] = set()
    extra_slugs: set[str] = set()

    index_path = os.path.join(CACHE_DIR, "index.json")
    if os.path.exists(index_path):
        with open(index_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        for entry in data.get("pages", []):
            url = entry.get("url")
            slug = entry.get("slug")
            path = entry.get("path")
            if isinstance(url, str) and isinstance(slug, str) and isinstance(path, str):
                pages.append(Page(url=url, slug=slug, path=path))
                visited_urls.add(url)
                visited_slugs.add(slug)
                if slug in LISTING_SLUGS and os.path.exists(path):
                    try:
                        with open(path, "r", encoding="utf-8") as html_file:
                            html = html_file.read()
                        if slug == "Jokers":
                            joker_slugs.update(extract_joker_slugs(html))
                        extra_slugs.update(extract_listing_slugs(html))
                    except Exception:
                        pass

    for seed in SEEDS:
        url = normalize_url(seed)
        if url:
            queue.append((url, 0))

    for slug in sorted(joker_slugs):
        url = normalize_url(f"/w/{slug}")
        if url and url not in visited_urls:
            queue.append((url, 0))

    while queue:
        url, depth = queue.pop(0)
        if url in visited_urls:
            continue
        visited_urls.add(url)

        try:
            html = fetch(url)
        except Exception as exc:
            print(f"failed: {url} ({exc})")
            continue

        parsed = urlparse(url)
        slug = slug_from_path(parsed.path)
        if slug in visited_slugs:
            print(f"skip duplicate slug: {slug}")
            time.sleep(SLEEP_SECONDS)
            continue

        page = Page(url=url, slug=slug, path="")
        save_page(page, html)
        pages.append(page)
        visited_slugs.add(slug)
        print(f"saved: {slug}")

        if slug in LISTING_SLUGS:
            new_slugs = set()
            if slug == "Jokers":
                new_slugs.update(extract_joker_slugs(html))
            new_slugs.update(extract_listing_slugs(html))
            added = 0
            for item_slug in sorted(new_slugs):
                if item_slug in joker_slugs or item_slug in extra_slugs:
                    continue
                if is_joker_slug(item_slug, joker_slugs):
                    joker_slugs.add(item_slug)
                else:
                    extra_slugs.add(item_slug)
                url = normalize_url(f"/w/{item_slug}")
                if url and url not in visited_urls:
                    queue.append((url, depth + 1))
                    added += 1
            if added == 0:
                parser = LinkParser()
                parser.feed(html)
                for href in parser.links:
                    normalized = normalize_url(href)
                    if not normalized:
                        continue
                    link_slug = slug_from_path(urlparse(normalized).path)
                    if not allowed_slug(link_slug, joker_slugs, extra_slugs):
                        continue
                    if normalized not in visited_urls:
                        queue.append((normalized, depth + 1))

        time.sleep(SLEEP_SECONDS)

    index = {
        "base": BASE,
        "count": len(pages),
        "pages": [{"url": p.url, "slug": p.slug, "path": p.path} for p in pages],
    }
    with open(os.path.join(CACHE_DIR, "index.json"), "w", encoding="utf-8") as f:
        json.dump(index, f, ensure_ascii=False, indent=2)

    print(f"done: {len(pages)} pages")


if __name__ == "__main__":
    main()
