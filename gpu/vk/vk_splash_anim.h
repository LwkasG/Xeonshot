// (xeonshot) 60 FPS continuous-loop startup splash animation (Higgsfield AI + Xeon processor lightning arcs).
// Decompresses the 6x6 RGBA sprite-sheet atlas once into a Dear ImGui user texture and loops continuously throughout loading.
#pragma once

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <cstdint>

#include "vk_imgui.h"
#include "vk_ui.h"
#include "xeonshot_splash_atlas.h"
#include "zstd.h"

namespace mkwvk::splash {

namespace detail {

inline float clamp01(float x) { return std::clamp(x, 0.0f, 1.0f); }
inline float lin01(float t, float t0, float t1) { return t1 > t0 ? clamp01((t - t0) / (t1 - t0)) : (t >= t0 ? 1.0f : 0.0f); }
inline float ease_out_cubic(float u) {
    const float k = 1.0f - clamp01(u);
    return 1.0f - k * k * k;
}

inline void frame_uv(int frame_idx, ImVec2* uv0, ImVec2* uv1) {
    const int idx = ((frame_idx % kAtlasFrames) + kAtlasFrames) % kAtlasFrames;
    const int row = idx / kAtlasCols;
    const int col = idx % kAtlasCols;
    const float inv_w = 1.0f / (float)kAtlasW;
    const float inv_h = 1.0f / (float)kAtlasH;
    *uv0 = ImVec2((col * kAtlasTile + 0.5f) * inv_w, (row * kAtlasTile + 0.5f) * inv_h);
    *uv1 = ImVec2(((col + 1) * kAtlasTile - 0.5f) * inv_w, ((row + 1) * kAtlasTile - 0.5f) * inv_h);
}

inline ImTextureData* ensure_atlas_texture() {
    static ImTextureData* tex = nullptr;
    static bool tried = false;
    if (!tried) {
        tried = true;
        ImTextureData* candidate = ui::user_texture(kAtlasW, kAtlasH);
        if (candidate && candidate->GetPixels()) {
            const size_t dec = ZSTD_decompress(candidate->GetPixels(), kAtlasRawBytes, kAtlasZstd, sizeof(kAtlasZstd));
            if (dec == kAtlasRawBytes) {
                ui::user_texture_changed(candidate);
                tex = candidate;
            }
        }
    }
    return tex;
}

} // namespace detail

inline float wordmark_alpha(float t) { return detail::ease_out_cubic(detail::lin01(t, 0.08f, 0.42f)); }
inline float flourish_progress(float t) { return detail::ease_out_cubic(detail::lin01(t, 0.14f, 0.52f)); }
inline float version_alpha(float t) { return detail::ease_out_cubic(detail::lin01(t, 0.22f, 0.60f)); }

inline void draw_background(ui::Ctx& c, float t, float cx, float cy, float lh) {
    const float s = c.s, w = c.w, h = c.h;
    // Deep Xeon midnight navy backdrop matching the emblem's native palette
    c.d->AddRectFilledMultiColor(ImVec2(0, 0), ImVec2(w, h),
                                 ui::rgba(5, 11, 26), ui::rgba(5, 11, 26),
                                 ui::rgba(8, 17, 38), ui::rgba(8, 17, 38));

    // Subtle Xeon circuit trace lattice with traveling data pulses
    const float step = std::max(72.0f, 96.0f * s);
    const ImU32 trace_col = ui::rgba(22, 168, 224, 15);
    const ImU32 pulse_col = ui::rgba(56, 224, 255, 32);
    const int nx = (int)(w / step) + 2;
    const int ny = (int)(h / step) + 2;
    for (int ix = -nx / 2; ix <= nx / 2; ix++) {
        const float x = cx + ix * step;
        if (x < 0 || x > w) continue;
        c.d->AddLine(ImVec2(x, 0), ImVec2(x, h), trace_col, std::max(1.0f, 0.9f * s));
        const float py = std::fmod(t * 140.0f * s + (ix & 3) * 190.0f * s, h + 160.0f * s) - 80.0f * s;
        c.d->AddLine(ImVec2(x, std::max(0.0f, py)), ImVec2(x, std::min(h, py + 44.0f * s)), pulse_col, std::max(1.2f, 1.4f * s));
    }
    for (int iy = -ny / 2; iy <= ny / 2; iy++) {
        const float y = cy + iy * step;
        if (y < 0 || y > h) continue;
        c.d->AddLine(ImVec2(0, y), ImVec2(w, y), trace_col, std::max(1.0f, 0.9f * s));
    }

    // Breathing layered radial cyan halo synchronized with the 3.2s loop cycle
    constexpr float kLoopSecs = 3.2f;
    const float phase = 2.0f * 3.14159265f * std::fmod(std::max(0.0f, t), kLoopSecs) / kLoopSecs;
    const float breathe = 1.0f + 0.08f * (0.5f - 0.5f * std::cos(phase));
    const float rad = std::max(lh * 1.45f, h * 0.24f) * breathe;
    for (int i = 28; i >= 1; i--) {
        const float frac = (float)i / 28.0f;
        c.d->AddCircleFilled(ImVec2(cx, cy + 8 * s), rad * frac, ui::rgba(24, 186, 240, 5), 96);
    }
}

inline void draw_emblem(ui::Ctx& c, float t, float cx, float cy, float lh, ImTextureData* logo, float aspect) {
    using namespace detail;
    const float entry_a = ease_out_cubic(lin01(t, 0.0f, 0.22f));
    ImTextureData* atlas = ensure_atlas_texture();
    if (atlas) {
        constexpr float kLoopSecs = 3.2f;
        const float loop_t = std::fmod(std::max(0.0f, t), kLoopSecs);
        const float f_pos = (loop_t / kLoopSecs) * (float)kAtlasFrames;
        const int f0 = (int)f_pos % kAtlasFrames;
        const int f1 = (f0 + 1) % kAtlasFrames;
        const float frac = f_pos - std::floor(f_pos);

        const float hw = lh * aspect * 0.5f * kAtlasQuadScale;
        const float hh = lh * 0.5f * kAtlasQuadScale;
        const ImVec2 p0(cx - hw, cy - hh), p1(cx + hw, cy + hh);

        ImVec2 uv0_a, uv1_a, uv0_b, uv1_b;
        frame_uv(f0, &uv0_a, &uv1_a);
        c.d->AddImage(atlas->GetTexRef(), p0, p1, uv0_a, uv1_a,
                      IM_COL32(255, 255, 255, (int)(255.0f * entry_a + 0.5f)));
        if (frac > 0.01f) {
            frame_uv(f1, &uv0_b, &uv1_b);
            c.d->AddImage(atlas->GetTexRef(), p0, p1, uv0_b, uv1_b,
                          IM_COL32(255, 255, 255, (int)(255.0f * entry_a * frac + 0.5f)));
        }
        return;
    }

    if (logo) {
        const float hw = lh * aspect * 0.5f;
        const float hh = lh * 0.5f;
        c.d->AddImage(logo->GetTexRef(), ImVec2(cx - hw, cy - hh), ImVec2(cx + hw, cy + hh),
                      ImVec2(0, 0), ImVec2(1, 1), IM_COL32(255, 255, 255, (int)(255.0f * entry_a + 0.5f)));
    }
}

} // namespace mkwvk::splash
