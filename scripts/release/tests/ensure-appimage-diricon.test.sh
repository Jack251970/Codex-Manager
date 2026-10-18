#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repair_script="$script_dir/../ensure-appimage-diricon.sh"
for tool in unsquashfs mksquashfs; do
  command -v "$tool" >/dev/null || { echo "$tool is required" >&2; exit 1; }
done

test_dir="$(mktemp -d)"
trap 'rm -rf -- "$test_dir"' EXIT
offset=512
passed=0

fail() { echo "FAIL: $*" >&2; exit 1; }

new_fixture() {
  case_dir="$test_dir/$1"
  app_dir="$case_dir/build/AppDir"
  image="$case_dir/fixture.AppImage"
  mkdir -p "$app_dir/usr/share/icons" "$app_dir/usr/share/applications"
  # The full one-pixel PNG keeps the fixture meaningful without image tools.
  printf '%s' 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aNF8AAAAASUVORK5CYII=' | base64 -d > "$app_dir/usr/share/icons/App.png"
  ln -s usr/share/icons/App.png "$app_dir/App.png"
  printf '[Desktop Entry]\nType=Application\nName=Fixture\nExec=fixture\nIcon=App\n' > "$app_dir/usr/share/applications/App.desktop"
  ln -s usr/share/applications/App.desktop "$app_dir/App.desktop"
}

pack_fixture() {
  # A tiny executable runtime exposes the same offset interface as AppImage.
  printf '#!/usr/bin/env bash\nif [[ "${1:-}" == --appimage-offset ]]; then echo %s; exit 0; fi\nexit 1\n' "$offset" > "$case_dir/runtime"
  truncate -s "$offset" "$case_dir/runtime"
  mksquashfs "$app_dir" "$case_dir/filesystem" -noappend -comp xz -quiet -processors 1 >/dev/null
  cat "$case_dir/runtime" "$case_dir/filesystem" > "$image"
  chmod 751 "$image"
  before_hash="$(sha256sum "$image" | cut -d ' ' -f1)"
}

repair_and_check() {
  local changed="$1" actual_mode
  bash "$repair_script" "$image" > "$case_dir/repair.log" 2>&1 || {
    cat "$case_dir/repair.log" >&2
    fail "repair failed for ${case_dir##*/}"
  }
  actual_mode="$(stat -c %a "$image")"
  [[ "$actual_mode" == 751 ]] || fail 'input mode changed'
  head -c "$offset" "$image" > "$case_dir/repaired-runtime"
  cmp "$case_dir/runtime" "$case_dir/repaired-runtime" || fail 'runtime changed'
  [[ "$("$image" --appimage-offset)" == "$offset" ]] || fail 'offset changed'
  if [[ "$changed" == false ]]; then
    [[ "$(sha256sum "$image" | cut -d ' ' -f1)" == "$before_hash" ]] || fail 'valid image was repacked'
  fi
  local repaired_hash
  repaired_hash="$(sha256sum "$image" | cut -d ' ' -f1)"
  bash "$repair_script" "$image" >> "$case_dir/repair.log" 2>&1 || fail 'second verification failed'
  [[ "$(sha256sum "$image" | cut -d ' ' -f1)" == "$repaired_hash" ]] || fail 'repair is not idempotent'
  # Removing the build directory reproduces the catalog's isolated environment.
  mv "$case_dir/build" "$case_dir/build-hidden"
  unsquashfs -quiet -offset "$offset" -dest "$case_dir/check" "$image" >/dev/null
  [[ -f "$case_dir/check/.DirIcon" ]] || fail 'icon is unavailable outside build tree'
  cmp "$case_dir/check/.DirIcon" "$case_dir/build-hidden/AppDir/usr/share/icons/App.png" || fail 'icon contents changed'
  [[ -f "$case_dir/check/App.desktop" ]] || fail 'desktop entry is unavailable outside build tree'
  if [[ "$changed" == true ]]; then
    [[ "$(readlink "$case_dir/check/.DirIcon")" != /* ]] || fail 'absolute icon link remains'
  fi
  passed=$((passed + 1))
  echo "PASS: ${case_dir##*/}"
}

new_fixture valid-relative
ln -s App.png "$app_dir/.DirIcon"
pack_fixture
repair_and_check false

new_fixture valid-regular
cp "$app_dir/usr/share/icons/App.png" "$app_dir/.DirIcon"
pack_fixture
repair_and_check false

new_fixture missing
pack_fixture
repair_and_check true

new_fixture dangling
ln -s missing.png "$app_dir/.DirIcon"
pack_fixture
repair_and_check true

new_fixture absolute-existing
ln -s "$app_dir/App.png" "$app_dir/.DirIcon"
[[ -e "$app_dir/.DirIcon" ]] || fail 'absolute-link regression needs the build target to exist'
pack_fixture
repair_and_check true

new_fixture outside-relative
cp "$app_dir/usr/share/icons/App.png" "$case_dir/build/external.png"
ln -s ../external.png "$app_dir/.DirIcon"
[[ -e "$app_dir/.DirIcon" ]] || fail 'outside-link fixture target must exist'
pack_fixture
repair_and_check true

new_fixture absolute-intermediate
mkdir "$app_dir/portable"
cp "$app_dir/usr/share/icons/App.png" "$app_dir/portable/App.png"
ln -s portable/App.png "$app_dir/Portable.png"
ln -s "$app_dir/usr/share/icons" "$app_dir/build-icons"
rm "$app_dir/App.png"
ln -s build-icons/App.png "$app_dir/App.png"
ln -s App.png "$app_dir/.DirIcon"
[[ -e "$app_dir/.DirIcon" ]] || fail 'intermediate-link regression needs its target to exist'
pack_fixture
repair_and_check true
[[ "$(readlink "$case_dir/check/.DirIcon")" == Portable.png ]] || fail 'nonportable root PNG was selected'

new_fixture absolute-desktop
ln -s App.png "$app_dir/.DirIcon"
rm "$app_dir/App.desktop"
ln -s "$app_dir/usr/share/applications/App.desktop" "$app_dir/App.desktop"
pack_fixture
repair_and_check true
[[ "$(readlink "$case_dir/check/App.desktop")" == usr/share/applications/App.desktop ]] || fail 'desktop link was not normalized'

new_fixture no-portable-icon
rm "$app_dir/App.png"
ln -s "$app_dir/usr/share/icons/App.png" "$app_dir/App.png"
ln -s App.png "$app_dir/.DirIcon"
pack_fixture
if bash "$repair_script" "$image" > "$case_dir/repair.log" 2>&1; then
  fail 'accepted an image whose only root PNG relies on the build directory'
fi
[[ "$(sha256sum "$image" | cut -d ' ' -f1)" == "$before_hash" ]] || fail 'failed repair changed the input'
[[ "$(stat -c %a "$image")" == 751 ]] || fail 'failed repair changed input mode'
passed=$((passed + 1))
echo "PASS: no-portable-icon (input preserved)"

new_fixture verification-failure
pack_fixture
real_unsquashfs="$(command -v unsquashfs)"
mkdir "$case_dir/bin"
cat > "$case_dir/bin/unsquashfs" <<'SH'
#!/usr/bin/env bash
for arg in "$@"; do
  if [[ "$arg" == */verify ]]; then exit 97; fi
done
exec "$REAL_UNSQUASHFS" "$@"
SH
chmod +x "$case_dir/bin/unsquashfs"
if PATH="$case_dir/bin:$PATH" REAL_UNSQUASHFS="$real_unsquashfs" bash "$repair_script" "$image" > "$case_dir/repair.log" 2>&1; then
  fail 'accepted a failed independent extraction'
fi
[[ "$(sha256sum "$image" | cut -d ' ' -f1)" == "$before_hash" ]] || fail 'verification failure replaced the input'
[[ "$(stat -c %a "$image")" == 751 ]] || fail 'verification failure changed input mode'
[[ -z "$(find "$case_dir" -maxdepth 1 -name '.appimage-diricon.*' -print -quit)" ]] || fail 'temporary replacement was not cleaned up'
passed=$((passed + 1))
echo "PASS: verification-failure (input preserved)"

echo "Passed $passed AppImage metadata regression cases"
