/* (xeonshot) Windows cold-page working-set trimmer (HOS_COLD_TRIM): moves untouched guest heap and address-region
   pages to the Windows Standby List via VirtualUnlock, freeing gigabytes of physical RAM without discarding committed
   bytes or dropping large pages ([hos_large_heap_lo, hos_large_heap_hi)).

   Any page locked or still in use is left alone or backed off quickly; pages imported by the GPU (rx_guest_imported)
   are skipped so driver-pinned host-pointer imports are never disturbed.
   Default: on when physical RAM < 32 GB (and HOS_LARGE_PAGES=0); off on >= 32 GB unless HOS_COLD_TRIM=1.
   HOS_COLD_TRIM=0 disables it everywhere; HOS_COLD_TRIM=1 forces it even with >= 32 GB RAM or partial large pages. */
#ifndef HOS_COLD_TRIM_H
#define HOS_COLD_TRIM_H

#if defined(_WIN32) && !defined(A64_MEM_ZERO) && !defined(__EMSCRIPTEN__)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define COLD_TRIM_UNIT_SHIFT 16
#define COLD_TRIM_UNIT_BYTES (1ULL << COLD_TRIM_UNIT_SHIFT)
#define COLD_TRIM_PAGES_PER_UNIT 16
#define COLD_TRIM_TABLE_SIZE 131072u
#define COLD_TRIM_SETTLE_SECS 10
#define COLD_TRIM_SCAN_UNITS 1024     /* 64 MB inspected per pass */
#define COLD_TRIM_BUDGET_UNITS 512    /* 32 MB trimmed per pass at most */
#define COLD_TRIM_MAX_REGIONS 2048

typedef struct { void *va; ULONG_PTR attr; } ColdTrimWs;
typedef BOOL(WINAPI *ColdTrimQueryFn)(HANDLE, PVOID, DWORD);

typedef struct {
    uint64_t key;        /* (guest >> 16) + 1; 0 = empty */
    uint32_t seen_sec;   /* monotonic seconds when unit settled or backed off */
    uint8_t last_res;    /* resident 4 KB pages at last look (0..16) */
    uint8_t strikes;     /* times unit faulted back after trim */
    uint16_t skip_until; /* monotonic seconds until eligible again */
} ColdTrimSlot;

static ColdTrimSlot *cold_trim_slots;
static ColdTrimQueryFn cold_trim_ws;
static uint32_t cold_trim_sec_base_init;
static int64_t cold_trim_t0_ns;
static uint64_t cold_trim_heap_cur;
static int cold_trim_reg_idx;
static uint64_t cold_trim_reg_off;
static volatile LONG64 cold_trim_trimmed_units, cold_trim_bounced_units;

static uint32_t cold_trim_secs(void) {
    int64_t now = hos_now_ns();
    if (!cold_trim_sec_base_init) {
        cold_trim_t0_ns = now;
        cold_trim_sec_base_init = 1;
    }
    int64_t d = (now - cold_trim_t0_ns) / 1000000000LL;
    return d < 1 ? 1u : (uint32_t)d;
}

static ColdTrimSlot *cold_trim_slot(uint64_t unit_idx) {
    uint64_t key = unit_idx + 1ULL;
    uint32_t mask = COLD_TRIM_TABLE_SIZE - 1u;
    uint32_t h = (uint32_t)((key * 0x9E3779B97F4A7C15ULL) >> 47) & mask;
    ColdTrimSlot *oldest = &cold_trim_slots[h];
    for (uint32_t i = 0; i < 8; i++) {
        ColdTrimSlot *s = &cold_trim_slots[(h + i) & mask];
        if (!s->key || s->key == key) {
            if (!s->key) {
                memset(s, 0, sizeof *s);
                s->key = key;
            }
            return s;
        }
        if (s->seen_sec < oldest->seen_sec) oldest = s;
    }
    memset(oldest, 0, sizeof *oldest);
    oldest->key = key;
    return oldest;
}

/* Trims one 64 KB guest unit if resident, settled >= 10s without growth, not large-heap, and not GPU-imported. */
static int cold_trim_try_unit(uint64_t guest_addr, uint32_t now_s) {
    extern uint64_t hos_large_heap_lo, hos_large_heap_hi;
    extern int rx_guest_imported(uint64_t lo, uint64_t hi);
    if (hos_large_heap_hi > hos_large_heap_lo &&
        guest_addr < hos_large_heap_hi && guest_addr + COLD_TRIM_UNIT_BYTES > hos_large_heap_lo)
        return 0;

    MEMORY_BASIC_INFORMATION mbi;
    if (!VirtualQuery(GP(guest_addr), &mbi, sizeof mbi)) return 0;
    if (mbi.State != MEM_COMMIT || mbi.Protect != PAGE_READWRITE || mbi.Type != MEM_PRIVATE) return 0;
    if ((uintptr_t)mbi.BaseAddress + mbi.RegionSize < (uintptr_t)GP(guest_addr + COLD_TRIM_UNIT_BYTES)) return 0;

    ColdTrimWs info[COLD_TRIM_PAGES_PER_UNIT];
    for (int k = 0; k < COLD_TRIM_PAGES_PER_UNIT; k++) {
        info[k].va = GP(guest_addr + ((uint64_t)k << 12));
        info[k].attr = 0;
    }
    if (!cold_trim_ws(GetCurrentProcess(), info, sizeof info)) return 0;
    uint32_t resident = 0;
    for (int k = 0; k < COLD_TRIM_PAGES_PER_UNIT; k++) resident += (uint32_t)(info[k].attr & 1);

    ColdTrimSlot *s = cold_trim_slot(guest_addr >> COLD_TRIM_UNIT_SHIFT);
    if (s->strikes >= 4) return 0;
    if (s->skip_until && now_s < (uint32_t)s->skip_until) {
        s->last_res = (uint8_t)resident;
        return 0;
    }
    if (s->seen_sec && resident > s->last_res && s->last_res == 0) {
        /* Came back after a trim: back off 30s -> 60s -> 120s -> forever */
        cold_trim_bounced_units++;
        if (s->strikes < 4) s->strikes++;
        uint32_t wait_s = 30u << (s->strikes > 0 ? s->strikes - 1 : 0);
        uint32_t until = now_s + wait_s;
        s->skip_until = until > 0xFFFFu ? 0xFFFFu : (uint16_t)until;
        s->last_res = (uint8_t)resident;
        s->seen_sec = now_s;
        return 0;
    }
    if (!resident) {
        s->last_res = 0;
        s->seen_sec = now_s;
        return 0;
    }
    if (!s->seen_sec || resident > s->last_res) {
        s->seen_sec = now_s;
        s->last_res = (uint8_t)resident;
        return 0;
    }
    if (now_s - s->seen_sec < COLD_TRIM_SETTLE_SECS) return 0;
    if (rx_guest_imported(guest_addr, guest_addr + COLD_TRIM_UNIT_BYTES)) {
        s->seen_sec = now_s;
        return 0;
    }

    for (int k = 0; k < COLD_TRIM_PAGES_PER_UNIT;) {
        if (!(info[k].attr & 1)) { k++; continue; }
        int e = k;
        while (e < COLD_TRIM_PAGES_PER_UNIT && (info[e].attr & 1)) e++;
        VirtualUnlock(info[k].va, (SIZE_T)(e - k) << 12);
        k = e;
    }
    cold_trim_trimmed_units++;
    s->last_res = 0;
    s->seen_sec = now_s;
    return 1;
}

static DWORD WINAPI hos_cold_trim_thread(LPVOID unused) {
    (void)unused;
    SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
    uint64_t (*regs)[2] = (uint64_t (*)[2])malloc(sizeof(uint64_t[2]) * COLD_TRIM_MAX_REGIONS);
    if (!regs) return 0;
    uint32_t next_report = 60;
    for (;;) {
        Sleep(1000);
        extern uint64_t hos_heap_size, hos_large_heap_lo, hos_large_heap_hi;
        int hos_regions(uint64_t (*out)[2], int cap);
        const uint32_t now_s = cold_trim_secs();
        int scanned = 0, trimmed = 0;

        uint64_t heap_start = HOS_HEAP_BASE;
        if (hos_large_heap_hi > hos_large_heap_lo && hos_large_heap_hi > heap_start)
            heap_start = hos_large_heap_hi;
        const uint64_t heap_end = HOS_HEAP_BASE + hos_heap_size;
        if (heap_end > heap_start) {
            if (cold_trim_heap_cur < heap_start || cold_trim_heap_cur >= heap_end)
                cold_trim_heap_cur = heap_start;
            while (cold_trim_heap_cur + COLD_TRIM_UNIT_BYTES <= heap_end &&
                   scanned < COLD_TRIM_SCAN_UNITS / 2 && trimmed < COLD_TRIM_BUDGET_UNITS / 2) {
                scanned++;
                trimmed += cold_trim_try_unit(cold_trim_heap_cur, now_s);
                cold_trim_heap_cur += COLD_TRIM_UNIT_BYTES;
            }
            if (cold_trim_heap_cur + COLD_TRIM_UNIT_BYTES > heap_end)
                cold_trim_heap_cur = heap_start;
        }

        const int nregs = hos_regions(regs, COLD_TRIM_MAX_REGIONS);
        const int rcount = nregs < COLD_TRIM_MAX_REGIONS ? nregs : COLD_TRIM_MAX_REGIONS;
        if (rcount > 0) {
            if (cold_trim_reg_idx >= rcount) {
                cold_trim_reg_idx = 0;
                cold_trim_reg_off = 0;
            }
            while (scanned < COLD_TRIM_SCAN_UNITS && trimmed < COLD_TRIM_BUDGET_UNITS && cold_trim_reg_idx < rcount) {
                const uint64_t rlo = (regs[cold_trim_reg_idx][0] + COLD_TRIM_UNIT_BYTES - 1) & ~(COLD_TRIM_UNIT_BYTES - 1);
                const uint64_t rhi = regs[cold_trim_reg_idx][1] & ~(COLD_TRIM_UNIT_BYTES - 1);
                const uint64_t a = rlo + cold_trim_reg_off;
                if (a + COLD_TRIM_UNIT_BYTES > rhi) {
                    cold_trim_reg_idx++;
                    cold_trim_reg_off = 0;
                    continue;
                }
                scanned++;
                trimmed += cold_trim_try_unit(a, now_s);
                cold_trim_reg_off += COLD_TRIM_UNIT_BYTES;
            }
            if (cold_trim_reg_idx >= rcount) {
                cold_trim_reg_idx = 0;
                cold_trim_reg_off = 0;
            }
        }

        if (now_s >= next_report) {
            next_report = now_s + 60;
            if (cold_trim_trimmed_units > 0)
                hos_log("mem: cold trim: %lld MB unlocked from working set (%lld MB faulted back)",
                        (long long)(cold_trim_trimmed_units >> 4),
                        (long long)(cold_trim_bounced_units >> 4));
        }
    }
    return 0;
}

static void hos_cold_trim_init(void) {
    extern int hos_large_pages;
    if (hos_large_pages) {
        hos_log("mem: cold trim (HOS_COLD_TRIM): off (HOS_LARGE_PAGES covers the whole window)");
        return;
    }
    const char *e = getenv("HOS_COLD_TRIM");
    int want = -1;
    if (e && *e) {
        if (!strcmp(e, "0") || !strcmp(e, "off") || !strcmp(e, "false") || !strcmp(e, "no")) want = 0;
        else want = 1;
    } else {
        MEMORYSTATUSEX ms;
        ms.dwLength = sizeof ms;
        want = (GlobalMemoryStatusEx(&ms) && ms.ullTotalPhys < (32ULL << 30)) ? 1 : 0;
        if (!want) {
            hos_log("mem: cold trim (HOS_COLD_TRIM): off by default (%llu GB RAM; HOS_COLD_TRIM=1 enables)",
                    (unsigned long long)(ms.ullTotalPhys >> 30));
            return;
        }
    }
    if (!want) {
        hos_log("mem: cold trim (HOS_COLD_TRIM=0): off");
        return;
    }
    HMODULE k32 = GetModuleHandleA("kernel32.dll");
    cold_trim_ws = k32 ? (ColdTrimQueryFn)(void *)GetProcAddress(k32, "K32QueryWorkingSetEx") : NULL;
    if (!cold_trim_ws) {
        HMODULE ps = LoadLibraryA("psapi.dll");
        if (ps) cold_trim_ws = (ColdTrimQueryFn)(void *)GetProcAddress(ps, "QueryWorkingSetEx");
    }
    if (!cold_trim_ws) {
        hos_log("mem: cold trim (HOS_COLD_TRIM): unavailable (no QueryWorkingSetEx)");
        return;
    }
    cold_trim_slots = (ColdTrimSlot *)calloc(COLD_TRIM_TABLE_SIZE, sizeof(ColdTrimSlot));
    if (!cold_trim_slots) return;
    HANDLE th = CreateThread(NULL, 0, hos_cold_trim_thread, NULL, 0, NULL);
    if (th) {
        CloseHandle(th);
        hos_log("mem: cold trim (HOS_COLD_TRIM): on (moving settled heap/region pages to Windows standby; HOS_COLD_TRIM=0 disables)");
    } else {
        free(cold_trim_slots);
        cold_trim_slots = NULL;
    }
}
#else
static inline void hos_cold_trim_init(void) {}
#endif
#endif
