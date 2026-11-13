#!/usr/bin/env bash
# The reference, pcengine-python at v0.1.0, into external/ with a PyPy venv
# (PyPy runs it several times faster than CPython). Its CPU core,
# huc6280-python v0.3.0, comes with it (pinned in its pyproject).
# Usage: scripts/fetch_pcenginepython.sh [DEST] [SOURCE]
set -euo pipefail
PIN=v0.1.0
COMMIT=bc9b976f8b6ba29ff16fdcbd527978899d7e54b4  # what the tag names
ROOT=$(cd "$(dirname "$0")/.." && pwd)
DEST=${1:-"$ROOT/external/pcengine-python"}
SOURCE=${2:-https://github.com/alewman/pcengine-python.git}
PYTHON=${PYTHON:-pypy3}
if [ ! -d "$DEST/.git" ]; then
    git clone -q "$SOURCE" "$DEST"
fi
git -C "$DEST" fetch -q --tags origin 2>/dev/null || true
git -C "$DEST" checkout -q "$PIN"
[ "$(git -C "$DEST" rev-parse HEAD)" = "$COMMIT" ] || { echo "$PIN is not $COMMIT" >&2; exit 1; }
if [ ! -x "$DEST/.venv-pypy/bin/python" ]; then
    "$PYTHON" -m venv "$DEST/.venv-pypy"
fi
"$DEST/.venv-pypy/bin/python" -m pip install -q -e "$DEST"
echo "pcengine-python $(git -C "$DEST" describe --tags) in $DEST"
