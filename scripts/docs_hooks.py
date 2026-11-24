"""Integrate owner-approved artwork only when its source files are present."""
from pathlib import Path


def on_config(config):
    assets = Path(config.docs_dir) / "assets" / "brand"
    branding = {}
    for key, name in (("banner", "momokun-banner.webp"), ("mascot", "momokun-mascot.webp"),
                      ("social", "momokun-social.png")):
        if (assets / name).is_file():
            branding[key] = f"assets/brand/{name}"
    if "banner" not in branding and "social" in branding:
        branding["banner"] = branding["social"]
    for name in ("momokun-wordmark.webp", "momokun-wordmark.png"):
        if (assets / name).is_file():
            config.theme["logo"] = f"assets/brand/{name}"
            break
    if (assets / "favicon.png").is_file():
        config.theme["favicon"] = "assets/brand/favicon.png"
    config.extra["branding"] = branding
    return config
