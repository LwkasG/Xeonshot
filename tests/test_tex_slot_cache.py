"""Program texture+sampler slot cache checks (gpu/vk/vk_shader.h, gpu/vk/vk_draw.cpp build_descriptors).

No GPU or game: inspects the wired TexSlot path and extra_desc_counts caching rules.

    python -I -B tests/test_tex_slot_cache.py
"""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from srccut import block  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]


def main():
    shader_h = (ROOT / "gpu" / "vk" / "vk_shader.h").read_text(encoding="utf-8")
    if "mutable std::atomic<uint64_t> extra_desc_counts{~0ull};" not in shader_h:
        raise SystemExit("vk_shader.h: Program::extra_desc_counts must start as ~0")

    src = (ROOT / "gpu" / "vk" / "vk_draw.cpp").read_text(encoding="utf-8")
    build = block(src, "void build_descriptors(const NvnState& s, const Program& p) {")

    required = (
        ("p.extra_desc_counts.load(", "ntex and nwrites are read from Program::extra_desc_counts"),
        ("p.extra_desc_counts.store(", "ntex and nwrites are stored in Program::extra_desc_counts"),
        ("out.writes.reserve(nwrites);", "out.writes reserves nwrites ahead of descriptor loop"),
        ("sampler_book.watching.load(", "TexSlot cache steps aside when sampler_book is watching near the limit"),
        ("sgen_now", "TexSlot cache reads sampler_registrations_generation()"),
        ("sl->version == version && sl->samp_gen == sgen_now", "TexSlot hit verifies both texture version and sampler generation"),
        ("texture_gone_ring_on() && texture_kept_since(sl->tex, sl->destroy_gen, dgen_now)",
         "TexSlot hit refreshes destroy_gen via gone ring"),
        ("texture_touch_quick(tex, true)", "TexSlot hit touches cached texture once per submission epoch"),
        ("FINE_N(FC_TEX_HIT, 1);", "TexSlot hit increments FC_TEX_HIT"),
        ("FINE_N(FC_TEX_MISS, 1);", "TexSlot miss increments FC_TEX_MISS"),
        ("sampler && (view || !handle) && !null_tex_wanted(st, d)",
         "TexSlot miss never caches failed sampler, deferred/dummy non-zero handle, or RX_VK_NULL_TEX"),
    )
    for needle, why in required:
        if needle not in build:
            raise SystemExit("build_descriptors: " + why + " (missing: " + needle + ")")

    fp_at = build.find("desc_fp_note_version(handle, version, fp);")
    hit_at = build.find("if (slot_hit)")
    if fp_at < 0 or hit_at < 0 or fp_at >= hit_at:
        raise SystemExit("build_descriptors: desc_fp_note_version must run before TexSlot hit check")

    if "par_stats_on" in build or "par_why(" in build:
        raise SystemExit("build_descriptors: stale par_stats_on / par_why symbols present")

    print("PASS: TexSlot cache, extra_desc_counts, out.writes reserve, and invalidation guards are wired")


if __name__ == "__main__":
    main()
