<h1 align="center">
  <br>
  <a>
    <img src="assets/xeonshot.png" alt="Xeonshot" width="220">
  </a>
  <br>
  <b>Xeonshot</b>
  <br>
</h1>

<h4 align="center">
  Ocarina of Time nativo para Windows — otimizado e independente para plataformas "Xeon Bomba".<br>
  Native Windows port of Ocarina of Time — standalone and optimized for "Xeon Bomba" platforms.
</h4>

<p align="center">
  <a href="#português-brasil"><b>🇧🇷 Português (Brasil)</b></a> ·
  <a href="#english"><b>🇺🇸 English</b></a>
</p>

---

# Português (Brasil)

<p align="center">
  <a href="#objetivo-compatibilidade-com-xeon-bomba">Objetivo</a> |
  <a href="#o-que-foi-feito-no-xeonshot">O que foi feito</a> |
  <a href="#status-no-xeonshot">Medições e status</a> |
  <a href="#como-compilar-e-executar">Compilar e executar</a> |
  <a href="#documentação-créditos-e-licença">Documentação e créditos</a>
</p>

**Xeonshot** é um fork independente baseado no [Hookshot](https://git.notrexed.dev/Hookshot/Hookshot), focado em **compatibilidade, estabilidade e baixa latência no Windows para plataformas "Xeon Bomba"** (kits X99 com processadores **Intel Xeon E5 v3/v4 Haswell-EP / Broadwell-EP**, muitos núcleos físicos, clock single-core moderado e memória DDR4 multi-canal). **Todos os ajustes, otimizações, empacotamento e testes do Xeonshot foram focados em Windows** (Linux e Steam Deck não foram testados nem são o alvo deste fork). O projeto opera de forma autônoma com executável próprio (`Xeonshot.exe`), renderizador Vulkan próprio (`xeonshot_vk.dll`), arquivos próprios de configuração e log (`xeonshot_options.ini`, `xeonshot_env.txt`, `logs/xeonshot_log.txt`) e o construtor gráfico temático **XeonshotBuilder** (`XeonshotBuilder.exe`), mantendo compatibilidade com os switches de ambiente (`HOOKSHOT_*`, `HOS_*`, `RX_*`), mods e saves (`user/save`).

> [!IMPORTANT]
> Este repositório **não contém o jogo, chaves ou executável pronto**. Você precisa da sua própria cópia do jogo; tudo o que o build gera permanece exclusivamente no seu PC.

## Objetivo: compatibilidade com Xeon Bomba

Reduzir falhas de página/TLB, contenção entre anéis/threads em CPUs Xeon E5 v3/v4 no Windows e sobrecarga na compilação/execução do código recompilado, mantendo total fidelidade visual, áudio/controles e estabilidade com DLSS Frame Generation.

## O que foi feito no Xeonshot

1. **Large Pages de 2 MB direcionadas no heap (`HOS_LARGE_HEAP_MB`):**
   - Arquivos: [host/hos_main.c](host/hos_main.c), [host/mkw_glue.c](host/mkw_glue.c), [host/hos_mem.c](host/hos_mem.c), [gpu/vk/switches.def](gpu/vk/switches.def).
   - Reserva e faz commit direcionado do início do heap guest (`0x1000000000`, testado com `HOS_LARGE_HEAP_MB=7168` — `7168 MB` em páginas de `2048 KB`) via `SeLockMemoryPrivilege`, sem tentar travar a janela inteira de 512 GB e protegendo a fatia em páginas grandes contra chamadas `VirtualAlloc`/`VirtualProtect`/`VirtualFree` incompatíveis, com fallback automático seguro para páginas de 4 KB.
2. **`OotBarrier` otimizado para barramento dual-ring Broadwell-EP:**
   - Arquivos: [host/oot_barrier.h](host/oot_barrier.h), [host/oot_barrier.c](host/oot_barrier.c), [gpu/vk/switches.def](gpu/vk/switches.def), [tests/test_oot_barrier.py](tests/test_oot_barrier.py).
   - Isola o contador `waiters` por slot em linhas de cache separadas (`_Alignas(64) OotBarrierSlot`) e adiciona pré-spin configurável em espaço de usuário (`HOS_OOT_BARRIER_SPIN_US`) antes de ceder ao `WaitOnAddress`.
3. **Vetorização nativa SIMD/FMA3 no lifter (`nx2/fastvec.py`) e compilação guiada por perfil (PGO):**
   - Arquivo: [nx2/fastvec.py](nx2/fastvec.py), validado com [tests/test_vfpx_elem.py](tests/test_vfpx_elem.py) e [tests/test_vbytes.py](tests/test_vbytes.py) (`130M` checagens, `0` divergências).
   - Adiciona `a64_vfpx_nat_mul32_elem` e `a64_vfpx_nat_fma32_elem` (`_mm_fmadd_ps` FMA3 com fallback para NaN) com assinatura *by-value* sem escape de ponteiro, preservando `A64_LOCAL_VREGS=1` escalarizado em registradores `XMM` junto de compilação PGO e `-O3` nos top 37 chunks quentes para Broadwell.
4. **Execução standalone para Windows, splash animado em loop contínuo e XeonshotBuilder independente:**
   - Identidade visual temática Xeon + Zelda ([assets/xeonshot.png](assets/xeonshot.png), [assets/xeonshot_splash_loop.png](assets/xeonshot_splash_loop.png)), splash inicial animado em loop contínuo a 60 FPS durante todo o carregamento com luz azul da Triforce pulsando e raios elétricos saindo do processador Xeon ([gpu/vk/vk_splash_anim.h](gpu/vk/vk_splash_anim.h), [gpu/vk/xeonshot_splash_atlas.h](gpu/vk/xeonshot_splash_atlas.h)), runtime e logs isolados (`Xeonshot.exe`, `xeonshot_vk.dll`, `xeonshot_options.ini`, `xeonshot_env.txt`, `logs/xeonshot_log.txt`, caches `cache/xeonshot_*`) e **XeonshotBuilder** (`XeonshotBuilder.exe`) autônomo com pastas próprias.

## Status no Xeonshot

A tabela abaixo resume **apenas o que foi efetivamente executado e medido no Xeonshot** no hardware de referência (**Windows 11 + Intel Xeon E5-2686 v4 18C/36T + NVIDIA GeForce RTX 5080**). Registros históricos do upstream Hookshot foram removidos de [configurações testadas](docs/tested-configurations.md) e [docs/tested/](docs/tested/README.md).

| Item / Cenário | Resultado medido no Xeonshot |
|---|---|
| Stage 9A (`s9a-mkt-prec2`, Castle Town Market `3440×1440`, sem FG) | ✅ `46,06 FPS` médios (`21,71 ms`) |
| Stage 9A (`s9a-fld-recprio`, Hyrule Field `5120×2160`, sem FG) | ✅ `48,64 FPS` médios (`20,56 ms`) |
| Stage 9C (`s9c-mkt-lp7g`, Castle Town Market `3440×1440`, Large Pages 7 GB + `OotBarrier` + `fastvec` FMA3 + PGO, sem FG) | ✅ `45,72 FPS` médios (`21,87 ms`, `p50 21,73`, `p95 27,85`, `p99 33,10`, `max 44,89 ms`, `>=50 ms = 0`, `errs=0`) |
| Stage 9C (`s9c-fld-lp7g`, Hyrule Field `5120×2160`, Large Pages 7 GB + `OotBarrier` + `fastvec` FMA3 + PGO, sem FG) | ✅ `47,15 FPS` médios (`21,21 ms`, `p50 20,73`, `p95 26,04`, `p99 31,17`, `max 34,81 ms`, `>=50 ms = 0`, `errs=0`) |
| Ganho acumulado sobre a linha de base inicial Xeon (`~30,3–31,3 ms`) | ✅ Redução para `~20,6–21,9 ms` sem Frame Generation e `0` quadros `>= 50 ms` |
| DLSS Frame Generation com `RX_VK_PRESENT_QUEUE=0` | ✅ Validado interativamente sem `VK_ERROR_DEVICE_LOST`, com boa fluidez e baixa latência |
| Instalação standalone (`C:\Xeonshot\Xeonshot.exe`) | ✅ Validada com `SDL3` + controles SDL, `Vulkan 1.4` (`RTX 5080`), NVIDIA DLSS / Frame Generation e Large Pages `7168 MB` (`2048 KB`) |
| Menu in-game (`PT-BR`), controles, áudio, ultrawide e saves | ✅ Funciona no ambiente Windows Xeon de referência |
| Placas AMD, placas Intel, Linux e Steam Deck | ❓ Ainda não testados no Xeonshot |

Veja os registros individuais em [docs/tested-configurations.md](docs/tested-configurations.md) e [docs/tested/](docs/tested/README.md).

## Como compilar e executar

### Execução direta da instalação standalone no Windows

Basta abrir `C:\Xeonshot\Xeonshot.exe` diretamente pelo Windows Explorer ou pelo PowerShell:

```powershell
& 'C:\Xeonshot\Xeonshot.exe'
```

*(Caso queira sobrescrever switches apenas para a sessão atual, defina-os no PowerShell antes de chamar `C:\Xeonshot\Xeonshot.exe`.)*

| Arquivo / Pasta em `C:\Xeonshot` | Função |
|---|---|
| `Xeonshot.exe`, `xeonshot_vk.dll` | Executável principal do jogo e back-end Vulkan |
| `SDL2.dll`, `SDL3.dll` | Janela, áudio e suporte a controles via SDL3 |
| `ngx\nvngx_dlss.dll`, `ngx\nvngx_dlssg.dll` | Runtimes opcionais de NVIDIA DLSS e Frame Generation |
| `xeonshot_env.txt` | Variáveis de ambiente carregadas automaticamente ao iniciar |
| `xeonshot_options.ini`, `xeonshot_title.txt` | Opções do menu in-game e título personalizado da janela |
| `user\save` | Seus arquivos de save locais |
| `logs\xeonshot_log.txt`, `cache\` | Log de execução e caches isolados de shaders/programas |

### Preset Xeon Bomba (`xeonshot_env.txt`)

Salve o bloco abaixo como `xeonshot_env.txt` na mesma pasta de `Xeonshot.exe`:

```ini
RX_VK_ASYNC=1
RX_VK_RECORDER=thread
RX_VK_PARALLEL=0
RX_VK_DESCRIPTOR_SETS=0
HOS_MUTEX_IX=1
RX_VK_CAPTURE_STREAM=1
HOS_KEEP_FREED_PAGES=1
RX_VK_KEEP_FREED=1
RX_VK_POOL_IDLE=120
RX_VK_TARGET_POOL_MB=1536
RX_VK_MIRROR_SPARSE=0
HOS_CORES=8
HOS_HEAP_COMMIT=1
HOS_LARGE_HEAP_MB=7168
RX_VK_PRESENT_QUEUE=0
```

- `RX_VK_PRESENT_QUEUE=0` é recomendado na RTX 5080 ao usar DLSS Frame Generation com este preset.

> [!TIP]
> Para ativar as **Large Pages de 2 MB (`HOS_LARGE_HEAP_MB=7168`)** no Windows, conceda à sua conta de usuário a permissão **Bloquear páginas na memória** (*Lock pages in memory*) em `secpol.msc` → *Diretivas Locais* → *Atribuição de Direitos de Usuário* e reinicie ou faça logoff/login. Sem esse privilégio, o Xeonshot volta automaticamente para páginas de `4 KB`.

### Compilar a partir do código-fonte

Para gerar uma instalação standalone a partir do seu dump do jogo (`romfs` + `exefs` ou `.nup`), usando a cadeia de ferramentas (`llvm-mingw`, `emsdk`, `cpm`; `HOOKSHOT_NGX_SDK` opcional para DLSS) e mantendo `SDL2.dll` e `SDL3.dll` ao lado de `Xeonshot.exe`:

```powershell
python -I nx2/make_game.py --romfs <caminho\romfs> --exefs <caminho\exefs> --out C:\Xeonshot
# ou com pacote .nup:
python -I nx2/make_game.py --game <caminho\dump.nup> --out C:\Xeonshot
```

Para compilar o construtor gráfico **XeonshotBuilder** (requer `.NET 8 SDK`), gerando `XeonshotBuilder.exe`:

```powershell
dotnet build builder/HookshotBuilder/HookshotBuilder.csproj -c Release
```

## Documentação, créditos e licença

- **Documentação técnica e referências:** [docs/README.md](docs/README.md) · [Opções do menu no jogo](docs/in-game-menu.md) · [Guia de desenvolvimento](docs/development.md) · [Configurações testadas](docs/tested-configurations.md) · [CONTRIBUTING.md](CONTRIBUTING.md).
- **Créditos:** **Xeonshot** é baseado no projeto upstream **Hookshot** (**NotRexed** e contribuidores), com componentes de Eden, suyu, RenHDrum, `a64-kit`, `llvm-mingw`, NVIDIA DLSS SDK (opcional) e AMD FidelityFX SDK.
- **Licença:** software livre sob a licença [GNU General Public License v3 ou posterior](LICENSE) (`GPL-3.0-or-later`). Projeto não oficial de fãs, sem vínculo com a Nintendo.

---

# English

<p align="center">
  <a href="#goal-xeon-bomba-compatibility">Goal</a> |
  <a href="#what-was-built-in-xeonshot">What was built</a> |
  <a href="#project-status">Measurements &amp; status</a> |
  <a href="#how-to-build-and-run">Build &amp; run</a> |
  <a href="#documentation-credits-and-license">Docs &amp; credits</a>
</p>

**Xeonshot** is a standalone fork based on [Hookshot](https://git.notrexed.dev/Hookshot/Hookshot), focused specifically on **compatibility, stability, and low frame latency on Windows for "Xeon Bomba" platforms** (X99 kits with **Intel Xeon E5 v3/v4 Haswell-EP / Broadwell-EP** CPUs, high physical core counts, modest single-core clocks, and multi-channel DDR4). **All Xeonshot adjustments, optimizations, packaging, and testing are focused on Windows** (Linux and Steam Deck are untested and not targeted by this fork). It operates completely independently with its own executable (`Xeonshot.exe`), Vulkan renderer (`xeonshot_vk.dll`), configuration/log files (`xeonshot_options.ini`, `xeonshot_env.txt`, `logs/xeonshot_log.txt`), and themed GUI builder **XeonshotBuilder** (`XeonshotBuilder.exe`), while remaining compatible with environment switches (`HOOKSHOT_*`, `HOS_*`, `RX_*`), mods, and `user/save` saves.

> [!IMPORTANT]
> This repository **does not contain the game, keys, or prebuilt game binaries**. You need your own copy of the game; everything produced by the build stays strictly on your PC.

## Goal: Xeon Bomba compatibility

Reduce TLB/page-fault overhead, dual-ring thread contention on Xeon E5 v3/v4 CPUs on Windows, and recompiled-code execution overhead while preserving visual accuracy, audio/controller support, and stable DLSS Frame Generation.

## What was built in Xeonshot

1. **Targeted 2 MB Large Heap Pages (`HOS_LARGE_HEAP_MB`):**
   - Files: [host/hos_main.c](host/hos_main.c), [host/mkw_glue.c](host/mkw_glue.c), [host/hos_mem.c](host/hos_mem.c), [gpu/vk/switches.def](gpu/vk/switches.def).
   - Reserves and commits the initial guest heap slice (`0x1000000000`, tested at `HOS_LARGE_HEAP_MB=7168` — `7168 MB` backed by `2048 KB` large pages) via `SeLockMemoryPrivilege` without locking the full 512 GB address window, guarding the large-page slice from incompatible `VirtualAlloc`/`VirtualProtect`/`VirtualFree` calls with safe automatic fallback to 4 KB pages.
2. **`OotBarrier` tuned for Broadwell-EP dual-ring contention:**
   - Files: [host/oot_barrier.h](host/oot_barrier.h), [host/oot_barrier.c](host/oot_barrier.c), [gpu/vk/switches.def](gpu/vk/switches.def), [tests/test_oot_barrier.py](tests/test_oot_barrier.py).
   - Isolates `waiters` per slot in cache-line-aligned structs (`_Alignas(64) OotBarrierSlot`) and adds configurable user-space pre-spin (`HOS_OOT_BARRIER_SPIN_US`) before yielding to `WaitOnAddress`.
3. **Native SIMD/FMA3 vectorization in the lifter (`nx2/fastvec.py`) and PGO:**
   - File: [nx2/fastvec.py](nx2/fastvec.py), verified by [tests/test_vfpx_elem.py](tests/test_vfpx_elem.py) and [tests/test_vbytes.py](tests/test_vbytes.py) (`130M` checks, `0` mismatches).
   - Adds `a64_vfpx_nat_mul32_elem` and `a64_vfpx_nat_fma32_elem` (`_mm_fmadd_ps` FMA3 with NaN fallback) using a pointer-escape-free *by-value* signature so `A64_LOCAL_VREGS=1` stays scalarized in `XMM` registers alongside profile-guided optimization (PGO) and `-O3` on the top 37 hot chunks for Broadwell.
4. **Standalone Windows execution, continuous-loop animated splash, and independent XeonshotBuilder:**
   - Custom Xeon + Zelda visual theme ([assets/xeonshot.png](assets/xeonshot.png), [assets/xeonshot_splash_loop.png](assets/xeonshot_splash_loop.png)), 60 FPS continuous-loop startup animation throughout loading with pulsing Triforce energy and Xeon processor lightning arcs ([gpu/vk/vk_splash_anim.h](gpu/vk/vk_splash_anim.h), [gpu/vk/xeonshot_splash_atlas.h](gpu/vk/xeonshot_splash_atlas.h)), isolated runtime/config/log filenames (`Xeonshot.exe`, `xeonshot_vk.dll`, `xeonshot_options.ini`, `xeonshot_env.txt`, `logs/xeonshot_log.txt`, `cache/xeonshot_*` caches), and standalone themed **XeonshotBuilder** (`XeonshotBuilder.exe`).

## Project status

The table below lists **only what has actually been run and measured on Xeonshot** on the reference hardware (**Windows 11 + Intel Xeon E5-2686 v4 18C/36T + NVIDIA GeForce RTX 5080**). Historical upstream Hookshot hardware tests were removed from [tested configurations](docs/tested-configurations.md) and [docs/tested/](docs/tested/README.md).

| Item / Scenario | Measured Xeonshot status |
|---|---|
| Stage 9A (`s9a-mkt-prec2`, Castle Town Market `3440×1440`, FG off) | ✅ `46.06 fps` mean (`21.71 ms`) |
| Stage 9A (`s9a-fld-recprio`, Hyrule Field `5120×2160`, FG off) | ✅ `48.64 fps` mean (`20.56 ms`) |
| Stage 9C (`s9c-mkt-lp7g`, Castle Town Market `3440×1440`, 7 GB large pages + `OotBarrier` + `fastvec` FMA3 + PGO, FG off) | ✅ `45.72 fps` mean (`21.87 ms`, `p50 21.73`, `p95 27.85`, `p99 33.10`, `max 44.89 ms`, `>=50 ms = 0`, `errs=0`) |
| Stage 9C (`s9c-fld-lp7g`, Hyrule Field `5120×2160`, 7 GB large pages + `OotBarrier` + `fastvec` FMA3 + PGO, FG off) | ✅ `47.15 fps` mean (`21.21 ms`, `p50 20.73`, `p95 26.04`, `p99 31.17`, `max 34.81 ms`, `>=50 ms = 0`, `errs=0`) |
| Cumulative latency gain over early Xeon baseline (`~30.3–31.3 ms`) | ✅ Reduced to `~20.6–21.9 ms` without Frame Generation, `0` frames `>= 50 ms` |
| DLSS Frame Generation with `RX_VK_PRESENT_QUEUE=0` | ✅ Interactively verified without `VK_ERROR_DEVICE_LOST`, smooth pacing & low latency |
| Standalone installation (`C:\Xeonshot\Xeonshot.exe`) | ✅ Verified with `SDL3` + SDL controllers, `Vulkan 1.4` (`RTX 5080`), NVIDIA DLSS / Frame Generation, and `7168 MB` Large Pages (`2048 KB`) |
| In-game menu (`PT-BR`), controls, audio, ultrawide, and saves | ✅ Works on reference Windows Xeon system |
| AMD GPUs, Intel GPUs, Linux, and Steam Deck | ❓ Not tested on Xeonshot yet |

See full run records in [docs/tested-configurations.md](docs/tested-configurations.md) and [docs/tested/](docs/tested/README.md).

## How to build and run

### Launching the standalone Windows installation

Launch `C:\Xeonshot\Xeonshot.exe` directly from Windows Explorer or PowerShell:

```powershell
& 'C:\Xeonshot\Xeonshot.exe'
```

*(If needed, set per-session environment overrides in PowerShell before calling `C:\Xeonshot\Xeonshot.exe`.)*

| File / Folder in `C:\Xeonshot` | Purpose |
|---|---|
| `Xeonshot.exe`, `xeonshot_vk.dll` | Main game executable and Vulkan renderer |
| `SDL2.dll`, `SDL3.dll` | Windowing, audio, and SDL3 controller support |
| `ngx\nvngx_dlss.dll`, `ngx\nvngx_dlssg.dll` | Optional NVIDIA DLSS and Frame Generation runtimes |
| `xeonshot_env.txt` | Environment switches loaded automatically at startup |
| `xeonshot_options.ini`, `xeonshot_title.txt` | In-game menu settings and custom window title |
| `user\save` | Your local save files |
| `logs\xeonshot_log.txt`, `cache\` | Runtime log and isolated shader/program caches |

### Reference Xeon Bomba preset (`xeonshot_env.txt`)

Save this block as `xeonshot_env.txt` next to `Xeonshot.exe`:

```ini
RX_VK_ASYNC=1
RX_VK_RECORDER=thread
RX_VK_PARALLEL=0
RX_VK_DESCRIPTOR_SETS=0
HOS_MUTEX_IX=1
RX_VK_CAPTURE_STREAM=1
HOS_KEEP_FREED_PAGES=1
RX_VK_KEEP_FREED=1
RX_VK_POOL_IDLE=120
RX_VK_TARGET_POOL_MB=1536
RX_VK_MIRROR_SPARSE=0
HOS_CORES=8
HOS_HEAP_COMMIT=1
HOS_LARGE_HEAP_MB=7168
RX_VK_PRESENT_QUEUE=0
```

- `RX_VK_PRESENT_QUEUE=0` is recommended on RTX 5080 when running DLSS Frame Generation with this preset.

> [!TIP]
> To enable **2 MB Large Pages (`HOS_LARGE_HEAP_MB=7168`)** on Windows, grant your user account **Lock pages in memory** in `secpol.msc` → *Local Policies* → *User Rights Assignment* and reboot or sign out/in. Without this privilege, Xeonshot safely falls back to `4 KB` pages.

### Building from source

To build a standalone installation from your own dump (`romfs` + `exefs` or `.nup`), with the toolchain configured (`llvm-mingw`, `emsdk`, `cpm`; optional `HOOKSHOT_NGX_SDK` for DLSS) and `SDL2.dll`/`SDL3.dll` placed next to `Xeonshot.exe`:

```powershell
python -I nx2/make_game.py --romfs <path\to\romfs> --exefs <path\to\exefs> --out C:\Xeonshot
# or from a .nup package:
python -I nx2/make_game.py --game <path\to\dump.nup> --out C:\Xeonshot
```

To build the **XeonshotBuilder** GUI (requires `.NET 8 SDK`), producing `XeonshotBuilder.exe`:

```powershell
dotnet build builder/HookshotBuilder/HookshotBuilder.csproj -c Release
```

## Documentation, credits and license

- **Reference documentation:** [docs/README.md](docs/README.md) · [In-game menu options](docs/in-game-menu.md) · [Development guide](docs/development.md) · [Tested configurations](docs/tested-configurations.md) · [CONTRIBUTING.md](CONTRIBUTING.md).
- **Credits:** **Xeonshot** is built on upstream **Hookshot** (**NotRexed** and contributors), with components from Eden, suyu, RenHDrum, `a64-kit`, `llvm-mingw`, optional NVIDIA DLSS SDK, and AMD FidelityFX SDK.
- **License:** free software under the [GNU General Public License v3 or later](LICENSE) (`GPL-3.0-or-later`). Unofficial fan project, not affiliated with or endorsed by Nintendo.
