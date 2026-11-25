"""The parts XeonshotBuilder-Setup.exe downloads (builder/setup/setup.c), from a Xeonshot Builder package folder.

    python -I scripts/make_setup_parts.py <package folder> <tag> <out folder> [--base <url the parts are served from>]
                                          [--reuse <an earlier release's xeonshot-setup.txt>]

A release is split by how often each part changes, so an update downloads only what changed: the builder's own code (a few MB)
in every release, the .NET runtime, Python, emsdk, the CPM packages and third_party only when they change. llvm-mingw comes from
its official release. Each zip is deterministic (sorted names, fixed dates and attributes): a part whose files did not change
has the same SHA-256 in the next release, and the setup takes it from its download cache instead of downloading it again.

Writes <out>/xeonshot-setup.txt and one XeonshotBuilder-<part>-<sha256 prefix>.zip per part (upload the ones the builder-parts
release lacks). Scripts the build does not use (.ps1, .bat, .js,
.html: antivirus machine-learning guesses fire on them) are left out.

--reuse: a part whose tools/<name> folder the package lacks is taken from that list as it is (its line, with its URL) when it is
already hosted under --base with XeonshotBuilder naming; otherwise its zip is fetched and repacked into XeonshotBuilder/ under --base.
"""
import argparse
import concurrent.futures
import hashlib
import os
import sys
import tempfile
import urllib.error
import urllib.request
import zipfile

SKIP_EXT = {".ps1", ".psm1", ".bat", ".cmd", ".js", ".vbs", ".html", ".htm", ".hta", ".wsf", ".pyc"}
SKIP_DIRS = {"__pycache__"}
TOOLS_PARTS = ("python", "emsdk", "cpm", "third_party", "wgpu-native")   # (tools/<name>: each its own part)
LLVM = ("552191b5754c49a5a20723276a6fc56541ea1922c4c50b7b669022333fc1b40f", 191037046,
        "llvm-mingw-20261104-ucrt-x86_64",
        "https://github.com/mstorsjo/llvm-mingw/releases/download/20261104/llvm-mingw-20261104-ucrt-x86_64.zip")


def files_under(root, top):
    out = []
    for d, ds, fs in os.walk(os.path.join(root, top)):
        ds[:] = sorted(x for x in ds if x not in SKIP_DIRS)
        for f in fs:
            if os.path.splitext(f)[1].lower() not in SKIP_EXT:
                out.append(os.path.relpath(os.path.join(d, f), root).replace(os.sep, "/"))
    return out


def write_zip(path, root, names):
    """Deterministic: the same files give the same bytes (sorted, a fixed date, fixed attributes)"""
    with zipfile.ZipFile(path + ".new", "w", zipfile.ZIP_DEFLATED, compresslevel=6) as z:
        for n in sorted(names):
            info = zipfile.ZipInfo("XeonshotBuilder/" + n, date_time=(2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            with open(os.path.join(root, n.replace("/", os.sep)), "rb") as f:
                z.writestr(info, f.read(), compresslevel=6)
    os.replace(path + ".new", path)
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while b := f.read(1 << 20):
            h.update(b)
    return h.hexdigest(), os.path.getsize(path)


def repack_reused_part(line, name, out_dir, base):
    """Fetch a legacy/external part zip and repack it deterministically under XeonshotBuilder/."""
    w = line.split()
    sha, size, url = w[1], int(w[2]), w[6]
    if not url.startswith("https://"):
        raise ValueError("not an https link: " + url)
    with tempfile.NamedTemporaryFile(suffix=".zip", delete=False) as tmp:
        tmp_path = tmp.name
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "xeonshot-ci"})
        with urllib.request.urlopen(req, timeout=180) as resp, open(tmp_path, "wb") as f:
            f.write(resp.read())
        h = hashlib.sha256()
        with open(tmp_path, "rb") as f:
            while b := f.read(1 << 20):
                h.update(b)
        if os.path.getsize(tmp_path) != size or h.hexdigest() != sha:
            sys.exit("%s: not the size and SHA-256 the list gives" % url)
        zp = os.path.join(out_dir, "XeonshotBuilder-%s.zip" % name)
        with zipfile.ZipFile(tmp_path, "r") as zin, zipfile.ZipFile(zp + ".new", "w", zipfile.ZIP_DEFLATED, compresslevel=6) as zout:
            entries = []
            for m in zin.infolist():
                if m.is_dir():
                    continue
                fn = m.filename
                if fn.startswith("HookshotBuilder/"):
                    fn = "XeonshotBuilder/" + fn[len("HookshotBuilder/"):]
                elif not fn.startswith("XeonshotBuilder/"):
                    fn = "XeonshotBuilder/" + fn.lstrip("/")
                entries.append((fn, m.filename))
            for fn, orig in sorted(entries):
                info = zipfile.ZipInfo(fn, date_time=(2026, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o644 << 16
                zout.writestr(info, zin.read(orig), compresslevel=6)
        os.replace(zp + ".new", zp)
        nh = hashlib.sha256()
        with open(zp, "rb") as f:
            while b := f.read(1 << 20):
                nh.update(b)
        new_sha, new_size = nh.hexdigest(), os.path.getsize(zp)
        named = os.path.join(out_dir, "XeonshotBuilder-%s-%s.zip" % (name, new_sha[:12]))
        os.replace(zp, named)
        return "part %s %d . - - %s/%s" % (new_sha, new_size, base, os.path.basename(named)), len(entries), new_size, new_sha
    finally:
        if os.path.exists(tmp_path):
            os.remove(tmp_path)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("package")
    p.add_argument("tag")
    p.add_argument("out")
    p.add_argument("--base", help="where the parts are served from (default: this repository's builder-parts release)")
    p.add_argument("--reuse", help="an earlier xeonshot-setup.txt: the parts the package has no folder for, from it")
    a = p.parse_args()
    root = os.path.abspath(a.package)
    if not os.path.isfile(os.path.join(root, "XeonshotBuilder.exe")):
        sys.exit("not a Xeonshot Builder package folder: %s" % root)
    # (the parts live in one release of their own, named by their checksum: a part that did not change is not uploaded again, and a
    # release itself holds only XeonshotBuilder.exe (the setup) and the list it reads)
    base = (a.base or "https://github.com/LwkasG/Xeonshot/releases/download/builder-parts").rstrip("/")
    os.makedirs(a.out, exist_ok=True)
    with open(os.path.join(root, "tools", "version.txt"), "w", newline="\n") as f:
        f.write(a.tag + "\n")

    top = sorted(os.listdir(root))
    builder_files = [n for n in top if os.path.isfile(os.path.join(root, n)) and
                     (n.startswith("XeonshotBuilder") or n in ("LICENSE", "README.txt"))]   # (XeonshotBuilder-Setup.exe too: Repair runs it)
    runtime_files = [n for n in top if os.path.isfile(os.path.join(root, n)) and n not in builder_files]
    runtime_dirs = [n for n in top if os.path.isdir(os.path.join(root, n)) and n != "tools"]
    tools = sorted(os.listdir(os.path.join(root, "tools")))
    parts = {
        "app": builder_files + [n for t in tools if t not in TOOLS_PARTS and t != "llvm-mingw"
                                for n in (["tools/" + t] if os.path.isfile(os.path.join(root, "tools", t)) else files_under(root, "tools/" + t))],
        "dotnet": runtime_files + [n for d in runtime_dirs for n in files_under(root, d)],
    }
    for t in TOOLS_PARTS:
        if os.path.isdir(os.path.join(root, "tools", t)):
            parts[t] = files_under(root, "tools/" + t)
    reuse = {}
    if a.reuse:
        with open(a.reuse, encoding="utf-8") as f:
            for line in f.read().splitlines():
                w = line.split()
                leaf = w[6].rsplit("/", 1)[-1] if len(w) == 7 and w[0] == "part" else ""
                for pre in ("XeonshotBuilder-", "HookshotBuilder-"):
                    if leaf.startswith(pre):
                        reuse[leaf[len(pre):].rsplit("-", 1)[0]] = line
                        break
    lines = ["tag %s" % a.tag]
    names = ("app", "dotnet") + TOOLS_PARTS
    for name in names:
        if not parts.get(name) and a.reuse and name in TOOLS_PARTS and name not in reuse:
            sys.exit("no tools/%s in the package and no %s part in %s" % (name, name, a.reuse))
    # (the parts zipped at once, one thread each, as scripts/linux/make_parts.py: zlib lets go of the GIL while it compresses, and
    # each zip is the same bytes as when they are made one after the other. The list keeps this order)
    todo = [n for n in names if parts.get(n)]
    with concurrent.futures.ThreadPoolExecutor(max(1, min(len(todo), os.cpu_count() or 1))) as pool:
        zipped = dict(zip(todo, pool.map(lambda n: write_zip(os.path.join(a.out, "XeonshotBuilder-%s.zip" % n), root, parts[n]),
                                         todo)))
    for name in names:
        if not parts.get(name):
            if a.reuse and name in TOOLS_PARTS:
                line = reuse[name]
                url = line.split()[6]
                leaf = url.rsplit("/", 1)[-1]
                if leaf.startswith("XeonshotBuilder-") and url.startswith(base + "/"):
                    lines.append(line)
                    print("%-12s (as in %s)" % (name, os.path.basename(a.reuse)), flush=True)
                else:
                    repacked, nfiles, rsize, rsha = repack_reused_part(line, name, a.out, base)
                    lines.append(repacked)
                    print("%-12s %5d files %5d MB  %s (repacked from %s)" % (name, nfiles, rsize >> 20, rsha[:16], os.path.basename(a.reuse)), flush=True)
            continue
        zp = os.path.join(a.out, "XeonshotBuilder-%s.zip" % name)
        sha, size = zipped[name]
        named = os.path.join(a.out, "XeonshotBuilder-%s-%s.zip" % (name, sha[:12]))
        os.replace(zp, named)
        zp = named
        lines.append("part %s %d . - - %s/%s" % (sha, size, base, os.path.basename(zp)))
        print("%-12s %5d files %5d MB  %s" % (name, len(parts[name]), size >> 20, sha[:16]), flush=True)
    lines.append("part %s %d XeonshotBuilder/tools %s llvm-mingw %s" % LLVM)
    lines.append("run XeonshotBuilder/XeonshotBuilder.exe")
    with open(os.path.join(a.out, "xeonshot-setup.txt"), "w", newline="\n") as f:
        f.write("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
