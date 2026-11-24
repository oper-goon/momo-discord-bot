#!/usr/bin/env python3
"""Check generated HTML links, fragment targets, assets, and Pages path handling."""
from __future__ import annotations

import argparse
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urljoin, urlsplit
from urllib.request import Request, urlopen


class Document(HTMLParser):
    def __init__(self, content: str):
        super().__init__(convert_charrefs=True)
        self.ids: set[str] = set()
        self.links: list[str] = []
        self.feed(content)

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if attrs.get("id"):
            self.ids.add(attrs["id"])
        if tag == "a" and attrs.get("name"):
            self.ids.add(attrs["name"])
        for key in ("href", "src", "poster"):
            if attrs.get(key):
                self.links.append(attrs[key])


def validate(site: Path, site_url: str, external: bool = False) -> tuple[list[str], int]:
    site = site.resolve()
    documents = {path.resolve(): Document(path.read_text(encoding="utf-8"))
                 for path in site.rglob("*.html")}
    errors: list[str] = []
    base = urlsplit(site_url)
    prefix = base.path.rstrip("/") + "/"
    checked = 0
    external_urls: set[str] = set()
    for path, document in documents.items():
        page_url = site_url.rstrip("/") + "/" + path.relative_to(site).as_posix()
        for link in document.links:
            parsed = urlsplit(urljoin(page_url, link))
            if parsed.scheme not in {"http", "https"}:
                continue
            if parsed.netloc != base.netloc:
                if external:
                    external_urls.add(parsed._replace(fragment="").geturl())
                continue
            checked += 1
            if not parsed.path.startswith(prefix):
                errors.append(f"{path.relative_to(site)}: link escapes repository URL path: {link}")
                continue
            relative = unquote(parsed.path[len(prefix):])
            target = (site / relative).resolve()
            if not target.is_relative_to(site):
                errors.append(f"{path.relative_to(site)}: target escapes site: {link}")
                continue
            if target.is_dir():
                target /= "index.html"
            if not target.is_file():
                errors.append(f"{path.relative_to(site)}: missing target: {link}")
            elif parsed.fragment and target in documents:
                if unquote(parsed.fragment) not in documents[target].ids:
                    errors.append(f"{path.relative_to(site)}: missing fragment: {link}")
    if not documents:
        errors.append("No HTML documents found; build MkDocs first.")
    for url in sorted(external_urls):
        try:
            with urlopen(Request(url, headers={"User-Agent": "Momokun-docs-linkcheck/1.0"}), timeout=15) as response:
                if response.status >= 400:
                    errors.append(f"External link returned HTTP {response.status}: {url}")
        except Exception as error:
            # Never include response bodies or exception URLs/credentials in reports.
            errors.append(f"External link unavailable ({type(error).__name__}): {url}")
    return errors, checked


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--site-dir", type=Path, default=Path("site"))
    parser.add_argument("--site-url", default="https://oper-goon.github.io/momo-discord-bot/")
    parser.add_argument("--external", action="store_true", help="Also check external links (network required).")
    args = parser.parse_args()
    errors, checked = validate(args.site_dir, args.site_url, args.external)
    for error in errors:
        print(error)
    if errors:
        raise SystemExit(f"Documentation check failed: {len(errors)} issue(s).")
    print(f"Documentation links passed: {checked} internal links/assets checked.")


if __name__ == "__main__":
    main()
