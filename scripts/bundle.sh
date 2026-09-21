#!/usr/bin/env bash
set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
archive_name="nur-blog"
plugin_id="blog"
package_version="$(awk -F '"' '/^version = / { print $2; exit }' "$project_root/blog/Cargo.toml")"
archive_dir="$project_root/dist"
archive_path="$archive_dir/$archive_name-$package_version.tar.gz"
stage_dir="$(mktemp -d "${TMPDIR:-/tmp}/$archive_name.XXXXXX")"
package_dir="$stage_dir/$plugin_id"

cleanup() {
    rm -rf "$stage_dir"
}
trap cleanup EXIT

cd "$project_root"
cargo build --locked --package nur-blog --target wasm32-wasip2 --release

source_wasm="blog/target/wasm32-wasip2/release/nur_blog.wasm"
package_wasm="nur_blog.wasm"
for asset in \
    blog/assets/blog.min.css \
    blog/assets/blog.min.js \
    blog/assets/admin.min.css \
    blog/assets/admin.min.js; do
    test -s "$asset"
done
test -f blog/assets/theme-overrides.css
test -s blog/assets/favicon.svg

install -d "$package_dir/assets" "$package_dir/migrations"
install -m 0644 README.md LICENSE "$package_dir"
install -m 0644 blog/migrations/*.sql "$package_dir/migrations"
install -m 0644 \
    blog/assets/blog.min.css \
    blog/assets/blog.min.js \
    blog/assets/admin.min.css \
    blog/assets/admin.min.js \
    blog/assets/favicon.svg \
    blog/assets/theme-overrides.css \
    "$package_dir/assets"
install -m 0644 "$source_wasm" "$package_dir/$package_wasm"
sed "s#^module = .*#module = \"$package_wasm\"#" blog/plugin.toml > "$package_dir/plugin.toml"

mkdir -p "$archive_dir"
tar -C "$stage_dir" -czf "$archive_path" "$plugin_id"

printf 'Created %s\n' "$archive_path"
