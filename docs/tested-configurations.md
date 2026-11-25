# Tested configurations

What has actually been run on **Xeonshot**, where, and how it went. The README's
[status table](../README.md#project-status) summarizes tested and untested configurations; only runs executed on
Xeonshot are kept below.

Browse the [new run files](tested/) individually. For each new measured run,
add a self-contained `YYYY-MM-DD-topic-tester-run.md` file under `docs/tested/`, following the
[format and checking instructions](tested/README.md). The tester's name and a distinct run identifier are required;
repeat the context instead of saying `as above`. New results belong in those files, not in the table below.

<!-- tested-run-fragments -->

## Earlier runs (frozen)

Earlier Xeonshot Stage 9A baseline runs prior to Stage 9B/9C (`OotBarrier` split + 2 MB large pages + `fastvec` SIMD/FMA3).

| Date | Build | OS / desktop | GPU / driver | CPU / RAM | Settings | Where | Result |
|---|---|---|---|---|---|---|---|
| 2026-11-22 | xeonshot-stage9a (PGO + O3 top37 chunks + ENV9A) | Windows 11 | NVIDIA GeForce RTX 5080, unknown | Intel Xeon E5-2686 v4 (18C/36T), unknown RAM | 3440x1440, frame generation off, HOS_HEAP_COMMIT=1, HOS_CORES=8, RX_VK_ASYNC=1, RX_VK_RECORDER=thread, RX_VK_PARALLEL=0, RX_VK_DESCRIPTOR_SETS=0, HOS_MUTEX_IX=1, RX_VK_CAPTURE_STREAM=1, HOS_KEEP_FREED_PAGES=1, RX_VK_KEEP_FREED=1, RX_VK_POOL_IDLE=120, RX_VK_TARGET_POOL_MB=1536, RX_VK_MIRROR_SPARSE=0 | Castle Town Market (run s9a-mkt-prec2) | 46.06 fps (mean 21.71 ms); interactive DLSS Frame Generation also verified with RX_VK_PRESENT_QUEUE=0 without device lost |
| 2026-11-22 | xeonshot-stage9a (PGO + O3 top37 chunks + ENV9A) | Windows 11 | NVIDIA GeForce RTX 5080, unknown | Intel Xeon E5-2686 v4 (18C/36T), unknown RAM | 5120x2160, frame generation off, HOS_HEAP_COMMIT=1, HOS_CORES=8, RX_VK_ASYNC=1, RX_VK_RECORDER=thread, RX_VK_PARALLEL=0, RX_VK_DESCRIPTOR_SETS=0, HOS_MUTEX_IX=1, RX_VK_CAPTURE_STREAM=1, HOS_KEEP_FREED_PAGES=1, RX_VK_KEEP_FREED=1, RX_VK_POOL_IDLE=120, RX_VK_TARGET_POOL_MB=1536, RX_VK_MIRROR_SPARSE=0 | Hyrule Field (run s9a-fld-recprio) | 48.64 fps (mean 20.56 ms) |

## Limits of the current Xeonshot runs

- Only **Windows 11 + Intel Xeon E5-2686 v4 (18C/36T) + NVIDIA GeForce RTX 5080** has been measured and visually verified on Xeonshot so far (Castle Town Market `3440x1440` and Hyrule Field `5120x2160`).
- Upstream Hookshot test runs on other hardware (AMD Radeon GPUs, Intel Arc GPUs, Linux distributions, Steam Deck, RTX 30/40 series, and Ryzen/Core CPUs) were removed because they were not executed on Xeonshot and are currently untested here.
