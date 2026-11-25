# In-game menu options

Reference table of every option in the Insert (`F1`) menu (`gpu/vk/vk_menu.cpp`), verified by `tests/test_readme_menu_rows.py`.

| Tab | Option | Choices | Default |
|---|---|---|---|
| Graphics | Quality preset | Custom, Minimum (4GB), Low (6GB), Balanced (8GB), High (12GB), Ultra (16GB+), Auto (this PC) | Auto (this PC) on a first start; Custom in an options file from before it |
| | Aspect ratio | The shapes your screen's resolutions have, narrowest first (5:4, 4:3, 16:10, 16:9, 21:9, 32:9, ...), your monitor's own marked "(monitor's)" (after a restart) | Your monitor's |
| | Output resolution | The Aspect ratio's sizes: your screen's resolutions and the common ones (1440p, 4K, 5K at 16:9; 1920x1200, 2560x1600 at 16:10; 3440x1440 at 21:9; ...), and 2048x1152 (the game's own) at 16:9 (after a restart) | 2048x1152 |
| | Custom output size | A size of your own, typed as `<width>x<height>` (after a restart) | None |
| | Profile | Docked, Handheld (after a restart) | Docked |
| | Upscaler | Game's own, NVIDIA DLSS, NVIDIA DLAA (only in builds with NVIDIA's DLSS SDK, below; DLSS and DLAA on RTX GPUs), AMD FSR 3.1 (every build, on GPUs it runs on, below), AMD FSR 4, AMD FSR 4 Native AA (only in builds with FSR 4, below), Off (no upscaling) | Game's own |
| | DLSS model | Transformer (K) (CNN (E) in its place on cards with less than 7 GB of video memory), Transformer 2 (M), Transformer 2 (L), NVIDIA default (shown where DLSS runs; used by DLSS and DLAA) | Transformer (K); CNN (E) under 7 GB |
| | DLAA before the bloom | Off, On: where NVIDIA DLAA smooths the picture, after the game's bloom and tone map or before them (shown with NVIDIA DLSS or DLAA; changeable with the Upscaler on NVIDIA DLAA only; changes at once; `RX_VK_DLSS_PRE_BLOOM` over it for one run) | Off |
| | Render resolution | Auto (the game's own dynamic resolution), 100%, 75%, 67%, 50% (each with its size). With DLSS, FSR 3.1 or FSR 4: Auto, Quality (67%), Balanced (58%), Performance (50%), Ultra Performance (33%). With DLAA or FSR 4 Native AA: Native. With Off: Auto, 100%, 75%, 67%, 50%, 33% | Auto |
| | Sharpness | 0 to 100%, in steps of 10 | 0% |
| | Frame generation | Off, Auto (it shows who makes its frames: FSR 3.1 FG or Built-in FG), 2x to 8x; with DLSS FG up to what NVIDIA's runtime makes on the GPU (6x on an RTX 50, 2x on an RTX 40) | Off |
| | Frame generation by | Shown while Frame generation is on. DLSS FG (NVIDIA DLSS Frame Generation: RTX 40 and newer, with a temporal Upscaler), FSR 3.1 FG (AMD FSR 3.1's frame interpolation, from the scene's depth and motion: any GPU), Built-in FG. DLSS FG is offered only where it runs; elsewhere the row has FSR 3.1 FG and Built-in FG, and its line at FSR 3.1 FG says why not DLSS FG (the log has NVIDIA's answer). At DLSS FG, FSR 3.1 makes the frames DLSS FG does not (Frame generation's Auto). At DLSS FG, the game's own menus, movies and loading screens show only the game's own frames (they have no Upscaler picture to make frames from): the row's line and the FPS counter say DLSS FG paused. With an Upscaler that is not temporal, the built-in one makes the frames and the line says why. `RX_VK_DLSS_FG=1` fixes the row at DLSS FG for a run. Changes at once | DLSS FG where it runs, else FSR 3.1 FG (an options file's Auto from before is this default) |
| | Game frame rate cap | 60, 90, 120, 144, 165, 240 FPS, Uncapped, 30 FPS (after a restart) | 60 FPS |
| | NVIDIA Reflex | Off, On, On + Boost (shown only on NVIDIA GPUs with `VK_NV_low_latency2`) | Off |
| | AMD Anti-Lag 2 | Off, On (shown only on AMD GPUs with `VK_AMD_anti_lag`) | Off |
| | Fullscreen | Off, On (the same as <kbd>F11</kbd>) | Off |
| | Fullscreen monitor | Auto (current monitor), or a named connected screen. While windowed, chooses the next fullscreen destination; while fullscreen, moves the game. An unavailable choice is retained with a temporary fallback. | Auto (current monitor) |
| | V-Sync | Auto, On, Off | Auto |
| | Window scaling filter | Off, Catmull-Rom, Lanczos | Off |
| | CRT filter | Off, Lottes aperture grille, Lottes consumer TV, Hyllian Fast, zfast CRT (changes at once; Native HDR bypasses it) | Off |
| | CRT lines | 240, 480, 720 virtual lines (shown with a CRT filter on; changes at once; capped at half the destination height) | 240 |
| | Ultrawide menus | Full width; Centred, blurred sides; Centred, black bars (with the Aspect ratio at the monitor's shape) | Centred, blurred sides |
| | HUD layout | Screen's edges; Middle 16:9 (on a screen that is not 16:9; `HOS_WIDE_UI`, when set, decides) | Screen's edges |
| | HDR | Off, Expanded, Native (the game's) | Off |
| | HDR paper white | 80 to 400 nits (while HDR is on) | 200 nits |
| | HDR peak brightness | 400 to 4000 nits (while HDR is on) | 1000 nits |
| | HDR contrast | 50% to 200% around mid grey (while HDR is on) | 100% |
| | HDR tone mapper | Vanilla (the game's curve), PsychoV35 (RenoDX's; with HDR set to Native. DLSS, FSR and TAAU do not take its scene yet: the game's own upscale runs) | Vanilla |
| | Texture detail | Auto, Full, Medium, Low | Auto |
| | Anisotropic filtering | Game's, 2x, 4x, 8x, 16x | 16x |
| | Grass distance | Game's / 2x / 3x / 4x | Game's |
| | Shadow resolution | Auto (for the card's memory), Game's (1600), High (2200), Highest (3072): the sun's shadow maps, applies at once | Auto |
| | Foliage distance | 25% to 150% (400% uncapped), in steps of 25; applies immediately | 100% (original) |
| | Object detail | 100% to 250% (400% uncapped), in steps of 25; applies immediately | 100% (original) |
| | Grass density | None, 25% to 200% (300% uncapped), in steps of 25; applies immediately | 100% (original) |
| | Shadow distance | 25% to 200% (400% uncapped), in steps of 25; applies immediately | 100% (original) |
| | Contact shadows | On, Off | On |
| | Base quality | Normal, Low (handheld) | Normal |
| | GPU device | Auto, or each graphics card by name (after a restart) | Auto |
| Audio | Volume | 0 to 100%, in steps of 5 | 100% |
| | Mute in background | Off, On | Off |
| | Output device | Default or a named playback device (Windows, and Linux through SDL); applies live | Default |
| | Speaker layout | Auto, Stereo, 5.1, Mono (layout changes after a restart; Mono/Stereo live on stereo output) | Auto |
| | Microphone | Off, Default, or each microphone by name | Default |
| | Microphone gain | Software input level, 0–400%; applies to new samples without changing system volume | 100% |
| | Test microphone | Start/stop capture; live levels appear in the help box for Test microphone and Microphone gain | Stopped |
| Gameplay | Game speed | Per frame, Real time | Real time |
| | Field of view | 50% to 150%, in steps of 5; applies immediately during gameplay | 100% (original) |
| | Skip title screen | Off, On (after a restart) | Off |
| | Skip opening | Off, On (skips the new-game opening: the story movie and Navi waking Link; later cutscenes and the title screen stay) | Off |
| | Item wheel | Off, On (hold R or L for a wheel of your items: point with either stick, let go to equip) | Off |
| | Cycle items | Off, Last used, Basic, +Bombs, +Tools (a tap of R or L changes the item on ZR within its group) | Off |
| | Text speed | Normal, 2x, 4x, Instant | Normal |
| | Text size | Original, 110%, 125%, 150%, 175% (dialogue, Navi and signs) | Original |
| | Game language | The game's 16 languages, each by its own name (after a restart) | The one built with |
| | Menu language | Auto (the game's language), or English, Português (Brasil), Español (Latinoamérica), Español (España), Français, Deutsch, Italiano, Nederlands, Polski, Русский, 日本語, 한국어, 简体中文, 繁體中文 | Auto |
| | FPS counter | Off, FPS, Frame time, Graph, Breakdown, Dev | Off |
| | FPS counter position | Top right, Top left, Bottom right, Bottom left | Top right |
| | FPS counter size | Auto, Small, Medium, Large | Auto |
| | FPS counter details | Opens the FPS counter details page: Version and hardware at every level (Off, On; Off), then a line each of the Dev level (VRAM, Textures, Draws, Dispatches, Render passes, Draws skipped, Texture uploads, Pipelines built, GPU slot waits, Queue submits, Write-backs) and the CPU and GPU lines (Off, On; On) | Version and hardware off, every line on |
| | Menu size | Small, Medium, Large | Medium |
| | Discord status | Off, On ("Playing Xeonshot" and the time played in your Discord status, while Discord runs on this PC) | On |
| | Bug report | Saves a report into a new folder in `bug-reports` (below) | |
| Controls | Controls | Opens the Controls page | |
| | Controller in background | Off, On | Off |
| | Stick dead zone | Default (24%), 5%, 10%, 15%, 20%, 30% | Default (24%) |
| | Vibration | Auto, Basic, Off | Auto |
| | Vibration strength | 25% to 200%, in steps of 25 | 100% |
| | Test vibration | an action: Confirm buzzes the controller used last; the value says what plays it | Motors |
| | Motion | Opens the Motion page: Motion controls (Off, On; On), Motion sensitivity X (10% to 600%) and Y (10% to 300%; 100%), Invert motion X and Y (Off, On; Off), Motion deadzone (Low, Standard, High; Standard) | |
| | Camera speed | 50% to 400%, in steps of 25 | 100% |
| | Invert camera X | Off, On | Off |
| | Invert camera Y | Off, On | Off |
| | Disable camera centering | Off, On | Off |
| | Mouse camera | As the stick, Native | As the stick |
| | Mouse sensitivity | 0.10 to 5.00 | 1.00 |
| | Invert mouse vertical | Off, On | Off |
| | Invert mouse horizontal | Off, On | Off |
| | Auto run | Off, On | Off |
| | Walk mode | Hold, Toggle | Hold |
| | Walk speed | 15% to 90%, in steps of 5 | 45% |
| | Menu shortcut | Insert or F1; F1 only; Insert, F1 or Esc | Insert, F1 or Esc |
| | Controller menu button | None, Home, Back left (GL), Back right (GR) | Home |
| | Compact menu key | Right Shift, Right Ctrl | Right Shift |
| | Layout preset | Nintendo (Def), Xbox / PC, Swap Face, Switch 2 Pro, Custom (on the Controls page) | Nintendo (Def) |
| | Button prompts | Auto, Switch, Xbox, PlayStation 4, PlayStation 5, Steam Deck (on the Controls page) | Auto |
| NFC | NFC tag | The tag files of your NFC folder, by their names | The first |
| | Tap tag | Holds the chosen tag to the game's reader | |
| | Remove tag | Takes it off the reader | |
| | Scan tag on controller | Reads a tag you own with a Switch 2 Pro Controller (USB) and saves it in your NFC folder | |
| Debug | Debug tools | Off, On (the tools below it while on) | Off |
| | Developer: Frame pacing | Off, On | Off |
| | Developer: Threaded rendering | Auto, On, Off (after a restart). Auto is On with 6 or more processor cores for the game, and the render thread alone with 4 or 5 | Auto |
| | Developer: Descriptor reuse | Off, On (after a restart) | On |
| | Developer: Sampler cache | Off, On (after a restart) | Off |
| | Developer: Robust buffers | Auto, Off, On (after a restart) | Auto |
| | Developer: Uncap detail sliders | Off, On: the four detail rows above past their safe max | Off |
| | Developer: Present queue | Off, On (after a restart) | Off |
| | Developer: ASTC as RGBA8 | Off, On (after a restart) | Off |

## Notes

- **Threaded rendering:** On draws the picture on threads of its own (a render thread and a recorder thread) while
  the game goes on to its next frame, for a higher frame rate on a processor with threads to spare; it is still being
  tested and can show flashes. Off draws it on the game's own thread, as before. Auto chooses by the processor's cores
  for the game, each core counted once however many threads it runs: On with 6 or more, the render thread alone with 4
  or 5 (where the recorder thread beside it was slower), Off with fewer, and Off wherever the mode is not safe. It
  applies after a restart. `RX_VK_ASYNC` and `RX_VK_RECORDER`, when set, choose for one run.
- **Auto run:** the sprint goes on after the run button (A) is let go. Hold A for 0.4 s while moving (the roll, then
  the sprint, as usual), let go, and A stays held for the game while the stick stays pushed; a short press (a roll on
  its own) is not kept. Letting go of the stick or pressing A again stops it (a long hold keeps it again). Only while
  Link is yours to move: an event (a talk, a cutscene), the pause menu, the map, the Insert menu and the ocarina's notes
  end it, so A never picks an entry, skips text or plays a note there.
- **Walk key (`walk`):** keyboard-only binding in `xeonshot_options.ini` alongside `move_up`, `move_down`, `move_left`,
  `move_right`, `cam_up`, `cam_down`, `cam_left`, `cam_right`; **Walk mode** (`Hold` or `Toggle`) and **Walk speed**
  (`15%` to `90%`) control how `walk` behaves.

See the [graphics and controls guide](graphics-and-controls.md) for general menu behavior.
