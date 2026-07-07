#!/usr/bin/env sh
# Fetch a repo-local llama.cpp runtime for the Linux builder path.
set -eu
cd "$(dirname "$0")/.."

tag="${ADAD_LLAMA_CPP_RELEASE_TAG:-b9892}"
asset_include="${ADAD_LLAMA_CPP_ASSET_INCLUDE:-ubuntu}"
tools_root="build/tools/llama.cpp"
install_dir="$tools_root/$tag"
archive="$tools_root/llama-${tag}.archive"
api_url="https://api.github.com/repos/ggml-org/llama.cpp/releases/tags/$tag"

mkdir -p "$tools_root"
find_bin="find"
if [ -x /usr/bin/find ]; then
  find_bin=/usr/bin/find
fi

real_server_path="$(
  "$find_bin" "$install_dir" -type f -name 'llama-server' 2>/dev/null \
    | grep -v "^$install_dir/llama-server$" \
    | head -n 1
)"

write_wrapper() {
  server_path="$1"
  server_dir="$(dirname "$server_path")"
  server_dir_name="$(basename "$server_dir")"
  cat >"$install_dir/llama-server" <<EOF
#!/usr/bin/env sh
set -eu
script_dir="\$(CDPATH= cd -- "\$(dirname "\$0")" && pwd)"
runtime_dir="\$script_dir/$server_dir_name"
export LD_LIBRARY_PATH="\$runtime_dir\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}"
exec "\$runtime_dir/llama-server" "\$@"
EOF
  chmod +x "$install_dir/llama-server"
}

if [ -n "$real_server_path" ] && [ -f "$(dirname "$real_server_path")/libllama-server-impl.so" ]; then
  write_wrapper "$real_server_path"
  echo "llama runtime: ok"
  exit 0
fi

json="$(curl -fsSL "$api_url")"
download_url="$(
  printf '%s\n' "$json" \
    | sed -n 's/.*"browser_download_url":[[:space:]]*"\([^"]*\)".*/\1/p' \
    | grep "$asset_include" \
    | grep 'x64' \
    | grep -vE 'arm64|vulkan|rocm|openvino|sycl|cuda|hip|s390x' \
    | head -n 1
)"

[ -n "$download_url" ] || {
  echo "ERROR: could not locate a plain Ubuntu x64 CPU llama.cpp asset for tag '$tag'." >&2
  exit 1
}

rm -rf "$install_dir"
mkdir -p "$install_dir"
curl -fL "$download_url" -o "$archive"
if unzip -tqq "$archive" >/dev/null 2>&1; then
  unzip -oq "$archive" -d "$install_dir"
else
  tar -xzf "$archive" -C "$install_dir"
fi

server_path="$(
  "$find_bin" "$install_dir" -type f -name 'llama-server' \
    | grep -v "^$install_dir/llama-server$" \
    | head -n 1
)"
[ -n "$server_path" ] && [ -f "$(dirname "$server_path")/libllama-server-impl.so" ] || {
  echo "ERROR: downloaded llama.cpp runtime does not contain llama-server." >&2
  exit 1
}

write_wrapper "$server_path"

echo "llama runtime: ok"
