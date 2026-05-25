"""Pull every device's bundle-required files into the openlogi-gui crate.

Run this before `cargo bundle` (release packaging) so the resulting
`.app` ships with assets baked into `Contents/Resources/assets/`. The
runtime `AssetCache` will read those instead of hitting the network.

Behaviour:
- Fetches `<base>/index.json` (default `https://assets.openlogi.org`,
  overridable via `--base` / `OPENLOGI_ASSETS`).
- For each device entry, downloads `front_core.png` + `core_metadata.json`
  into `crates/openlogi-gui/bundle-assets/<depot>/`.
- Verifies sha256 against the index; skips files that already match.
- Removes orphaned depots (registry shrinkage).

bundle-assets/ is gitignored — every machine syncs its own copy.
"""

import argparse
import hashlib
import json
import os
import shutil
import sys
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
BUNDLE_DIR = REPO / "crates" / "openlogi-gui" / "assets"

DEFAULT_BASE = "https://assets.openlogi.org"
REQUIRED_FILES = ("front_core.png", "core_metadata.json")


def sha256_of(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(64 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


_UA = "openlogi-bundle-sync/1.0 (+https://github.com/AprilNEA/OpenLogi)"


def fetch(url: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": _UA})
    with urllib.request.urlopen(req, timeout=30) as resp:
        if resp.status != 200:
            raise SystemExit(f"GET {url} → HTTP {resp.status}")
        return resp.read()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--base",
        default=os.environ.get("OPENLOGI_ASSETS", DEFAULT_BASE),
        help="origin of the asset host (default: %(default)s)",
    )
    parser.add_argument(
        "--out",
        type=Path,
        default=BUNDLE_DIR,
        help="destination directory (default: %(default)s)",
    )
    args = parser.parse_args()

    base = args.base.rstrip("/")
    out: Path = args.out
    out.mkdir(parents=True, exist_ok=True)

    index_bytes = fetch(f"{base}/index.json")
    (out / "index.json").write_bytes(index_bytes)
    index = json.loads(index_bytes)
    devices = index["devices"]
    print(f"index.json: {len(devices)} devices")

    # Remove orphaned depots so the bundle stays in sync with the registry.
    expected = set(devices.keys())
    for child in out.iterdir():
        if child.is_dir() and child.name not in expected:
            print(f"  pruning {child.name}")
            shutil.rmtree(child)

    fetched = skipped = 0
    for depot, entry in sorted(devices.items()):
        dst_dir = out / depot
        dst_dir.mkdir(exist_ok=True)
        wanted = {f["name"]: f for f in entry["files"] if f["name"] in REQUIRED_FILES}
        missing = set(REQUIRED_FILES) - set(wanted)
        if missing:
            print(f"  WARN {depot}: registry missing {sorted(missing)}", file=sys.stderr)
            continue
        for name, file_meta in wanted.items():
            dst = dst_dir / name
            if dst.exists() and sha256_of(dst) == file_meta["sha256"]:
                skipped += 1
                continue
            url = f"{base}/{entry['asset_path']}{name}"
            dst.write_bytes(fetch(url))
            fetched += 1
            print(f"  {depot}/{name} ({file_meta['bytes']} B)")

    total_bytes = sum(
        (out / depot / name).stat().st_size
        for depot in devices
        for name in REQUIRED_FILES
        if (out / depot / name).exists()
    )
    print(
        f"done: {fetched} fetched, {skipped} cache-hit, "
        f"{total_bytes / 1024 / 1024:.1f} MB total under {out}"
    )


if __name__ == "__main__":
    main()
