//! The builder's windows, under `--output user`: a build's progress, an error, the crash window and its pages, and the
//! first run's picker of the game's .nup. Slint with only its winit back end and its software renderer (no Qt, no GPU:
//! a crash may have been the GPU's), two embedded fonts (Inter, and Noto Sans Black for headings), drawn in the Longshot Launcher's brand look (ui/windows.slint).
//!
//! The windows live on a thread of their own, which runs Slint's event loop (Slint's winit back end lets it run off the
//! main thread on Linux and Windows); [`Ui`] is a handle any thread calls. Nothing opens until a window is needed:
//!
//! - [`Ui::step`] shows the progress window only once a build has gone on for [`PROGRESS_AFTER`] (a quick update
//!   flashes nothing); [`Ui::done`] closes it, as soon as the game is started.
//! - [`Ui::page`], [`Ui::error`], [`Ui::crash_window`], [`Ui::pick_game`] wait for the player.
//! - A window that can't open (no display; winit fails) is [`NoWindow`]: the caller says it in text instead (and keeps
//!   a crash record for the next start).
//!
//! Keys held or mashed as a page appears press nothing: a key's repeat, and any key or click in a page's first
//! [`PAGE_GUARD`], are ignored.

use crash::flow::{Choice, Desk, Page, Pages};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The windows ui/windows.slint makes (compiled by build.rs).
#[allow(missing_docs, clippy::all)]
mod windows {
    slint::include_modules!();
}
use slint::{ComponentHandle, Model};
use windows::{PageWindow, PickWindow, ProgressWindow};

/// How long a build goes on before its progress window shows.
pub const PROGRESS_AFTER: Duration = Duration::from_millis(500);
/// A page's first moments, in which no key or click counts.
pub const PAGE_GUARD: Duration = Duration::from_millis(400);

/// Localized UI strings detected from the host system locale.
struct UiStrings {
    window_title: &'static str,
    installer_title: &'static str,
    pick_heading: &'static str,
    pick_sub: &'static str,
    pick_placeholder: &'static str,
    browse: &'static str,
    cancel: &'static str,
    install_and_play: &'static str,
    missing_file_note: &'static str,
    file_dialog_title: &'static str,
    progress_heading: &'static str,
    progress_sub: &'static str,
    total_label: &'static str,
    stages: [&'static str; 6],
    stage_descs: [&'static str; 6],
    stage_word: &'static str,
    of_word: &'static str,
    status_done: &'static str,
    status_active: &'static str,
    status_pending: &'static str,
    follow_active: &'static str,
    tip_badge: &'static str,
    tips: [&'static str; 5],
    toggle_log_show: &'static str,
    toggle_log_hide: &'static str,
    card_lang_title: &'static str,
    card_display_title: &'static str,
    card_preset_title: &'static str,
    lang_value: String,
    display_value: String,
    preset_value: &'static str,
    error_title: &'static str,
    close: &'static str,
}

fn strings() -> UiStrings {
    let raw_locale = build_core::system_locale().unwrap_or_else(|| "en-US".into());
    let family = raw_locale.split(['-', '_', '.', '@']).next().unwrap_or("en").to_ascii_lowercase();
    let (w, h) = build_core::primary_monitor_resolution().unwrap_or((1920, 1080));
    let ultrawide = w * 10 >= h * 20;
    let res_label = if ultrawide { format!("{w}x{h} (Ultrawide)") } else { format!("{w}x{h}") };
    match family.as_str() {
        "pt" => UiStrings {
            window_title: "Xeonshot",
            installer_title: "Xeonshot — Instalador",
            pick_heading: "Selecione a ROM (.nup) de Ocarina of Time",
            pick_sub: "O Xeonshot recompila o jogo nativamente a partir da sua própria cópia (.nup) de Nintendo Switch 2. Nada sai do seu PC.",
            pick_placeholder: "Caminho do arquivo .nup",
            browse: "Procurar...",
            cancel: "Cancelar",
            install_and_play: "Instalar e Jogar",
            missing_file_note: "Arquivo não encontrado. Cole o caminho do .nup ou clique em Procurar.",
            file_dialog_title: "Selecione o arquivo .nup de Ocarina of Time",
            progress_heading: "Instalando o Xeonshot",
            progress_sub: "O Xeonshot iniciará o jogo automaticamente quando terminar.",
            total_label: "TOTAL GLOBAL",
            stages: [
                "Preparando ferramentas e ambiente",
                "Lendo e verificando a ROM (.nup)",
                "Recompondo código do jogo (Lift)",
                "Compilando executável nativo",
                "Aplicando preset Xeonshot e assets",
                "Compilando shaders Vulkan",
            ],
            stage_descs: [
                "Extrai compilador LLVM, runtime Vulkan e ferramentas necessárias para o cache local.",
                "Lê e valida os arquivos da ROM (.nup) de Nintendo Switch 2 com verificação de integridade.",
                "Traduz as instruções originais do jogo (ARM64) para representação nativa LLVM IR.",
                "Compila todos os módulos C++/Rust otimizados e gera o binário nativo xeonshot.exe.",
                "Aplica idioma do Windows, resolução nativa do monitor e o preset validado Xeonshot + FG 3x.",
                "Pré-compila os pipelines e shaders Vulkan para eliminar engasgos durante a jogatina.",
            ],
            stage_word: "ETAPA",
            of_word: "DE",
            status_done: "Concluído",
            status_active: "Em andamento...",
            status_pending: "Aguardando",
            follow_active: "Seguir etapa atual",
            tip_badge: "DICA",
            tips: [
                "Clique em qualquer etapa acima para acompanhar o que o instalador está fazendo em detalhes.",
                "Pressione F1 ou Insert dentro do jogo para abrir o menu completo de gráficos, FPS e câmera.",
                "Seu jogo já nasce na resolução nativa do monitor com preset Xeonshot e Built-in FG 3x.",
                "100% privado e local: sua ROM (.nup) nunca sai do seu PC durante ou após a instalação.",
                "Saves e configurações ficam isolados na pasta profile/ e são preservados entre atualizações.",
            ],
            toggle_log_show: "Ver log ao vivo",
            toggle_log_hide: "Ocultar log",
            card_lang_title: "IDIOMA DETECTADO",
            card_display_title: "MONITOR NATIVO",
            card_preset_title: "PRESET DE FÁBRICA",
            lang_value: raw_locale,
            display_value: res_label,
            preset_value: "Xeonshot + FG 3x",
            error_title: "Xeonshot: não foi possível compilar o jogo",
            close: "Fechar",
        },
        "es" => UiStrings {
            window_title: "Xeonshot",
            installer_title: "Xeonshot — Instalador",
            pick_heading: "Selecciona la ROM (.nup) de Ocarina of Time",
            pick_sub: "Xeonshot recompila el juego de forma nativa desde tu propia copia (.nup) de Nintendo Switch 2. Nada sale de tu PC.",
            pick_placeholder: "Ruta del archivo .nup",
            browse: "Examinar...",
            cancel: "Cancelar",
            install_and_play: "Instalar y Jugar",
            missing_file_note: "No se encontró el archivo. Pega la ruta del .nup o haz clic en Examinar.",
            file_dialog_title: "Selecciona tu archivo .nup de Ocarina of Time",
            progress_heading: "Instalando Xeonshot",
            progress_sub: "Xeonshot iniciará el juego automáticamente al terminar.",
            total_label: "TOTAL GLOBAL",
            stages: [
                "Preparando herramientas y entorno",
                "Leyendo y verificando la ROM (.nup)",
                "Recompilando código del juego (Lift)",
                "Compilando ejecutable nativo",
                "Aplicando preset Xeonshot y assets",
                "Compilando shaders Vulkan",
            ],
            stage_descs: [
                "Prepara el compilador LLVM, el runtime Vulkan y las herramientas locales en caché.",
                "Lee y verifica la integridad de los datos de tu copia (.nup) de Nintendo Switch 2.",
                "Traduce el binario del juego (ARM64) a representación nativa LLVM IR.",
                "Compila todos los módulos optimizados y genera el ejecutable nativo xeonshot.exe.",
                "Aplica el idioma del sistema, la resolución nativa del monitor y el preset Xeonshot + FG 3x.",
                "Precompila los shaders Vulkan para evitar tirones durante la partida.",
            ],
            stage_word: "ETAPA",
            of_word: "DE",
            status_done: "Completado",
            status_active: "En curso...",
            status_pending: "En espera",
            follow_active: "Seguir etapa actual",
            tip_badge: "CONSEJO",
            tips: [
                "Haz clic en cualquier etapa arriba para inspeccionar qué ocurre en ese paso.",
                "Pulsa F1 o Insert dentro del juego para abrir el menú completo de gráficos y rendimiento.",
                "El juego se configura automáticamente a la resolución nativa de tu monitor + FG 3x.",
                "100% local y privado: tu archivo .nup nunca sale de tu ordenador.",
                "Tus partidas guardadas y ajustes se conservan en profile/ al actualizar.",
            ],
            toggle_log_show: "Ver registro en vivo",
            toggle_log_hide: "Ocultar registro",
            card_lang_title: "IDIOMA DETECTADO",
            card_display_title: "PANTALLA NATIVA",
            card_preset_title: "PRESET DE FÁBRICA",
            lang_value: raw_locale,
            display_value: res_label,
            preset_value: "Xeonshot + FG 3x",
            error_title: "Xeonshot: no se pudo compilar el juego",
            close: "Cerrar",
        },
        _ => UiStrings {
            window_title: "Xeonshot",
            installer_title: "Xeonshot — Installer",
            pick_heading: "Select your Ocarina of Time .nup ROM",
            pick_sub: "Xeonshot builds the game natively from your own copy of Ocarina of Time (Nintendo Switch 2): its .nup file. Nothing leaves this PC.",
            pick_placeholder: "Path to your .nup file",
            browse: "Browse...",
            cancel: "Cancel",
            install_and_play: "Install & Play",
            missing_file_note: "File not found. Paste the path to your .nup or click Browse.",
            file_dialog_title: "Select your Ocarina of Time .nup",
            progress_heading: "Installing Xeonshot",
            progress_sub: "Xeonshot will start the game automatically when it's done.",
            total_label: "OVERALL",
            stages: [
                "Preparing build tools and environment",
                "Reading and verifying game ROM (.nup)",
                "Lifting game code to native IR",
                "Compiling native executable",
                "Applying Xeonshot preset and assets",
                "Compiling Vulkan shaders",
            ],
            stage_descs: [
                "Sets up the LLVM toolchain, Vulkan runtime, and local build cache.",
                "Reads and validates game data from your own Nintendo Switch 2 (.nup) copy.",
                "Translates game instructions (ARM64) into native LLVM intermediate code.",
                "Compiles all optimized C++/Rust modules and links xeonshot.exe.",
                "Applies host locale, native monitor resolution, and the Xeonshot + FG 3x preset.",
                "Pre-compiles Vulkan shader pipelines for smooth, stutter-free gameplay.",
            ],
            stage_word: "STAGE",
            of_word: "OF",
            status_done: "Done",
            status_active: "In progress...",
            status_pending: "Pending",
            follow_active: "Follow current stage",
            tip_badge: "TIP",
            tips: [
                "Click any stage above to inspect what the installer does at that step.",
                "Press F1 or Insert in-game to open the full graphics, FPS, and camera settings menu.",
                "Configured automatically for your monitor's native resolution + Built-in FG 3x.",
                "100% local and private: your .nup copy never leaves this computer.",
                "Saves and settings live in profile/ and stay intact across future updates.",
            ],
            toggle_log_show: "Show live log",
            toggle_log_hide: "Hide log",
            card_lang_title: "SYSTEM LOCALE",
            card_display_title: "NATIVE DISPLAY",
            card_preset_title: "FACTORY PRESET",
            lang_value: raw_locale,
            display_value: res_label,
            preset_value: "Xeonshot + FG 3x",
            error_title: "Xeonshot: the game could not be built",
            close: "Close",
        },
    }
}

/// A question for the player ([`Ui::confirm`]): what is asked, the facts the answer should rest on, and the two answers.
#[derive(Clone, Copy, Debug)]
pub struct Confirm<'a> {
    /// The window's title.
    pub title: &'a str,
    /// The question, a paragraph each.
    pub lines: &'a [&'a str],
    /// What the player decides by, name and value: the mod, its source, its code's hash, the permissions asked.
    pub facts: &'a [(&'a str, &'a str)],
    /// The button that says yes.
    pub accept: &'a str,
    /// The button that says no.
    pub decline: &'a str,
}

impl Confirm<'_> {
    /// The facts as the window shows them, one a line, their values aligned.
    pub fn block(&self) -> String {
        let width = self.facts.iter().map(|(k, _)| k.chars().count()).max().unwrap_or(0);
        self.facts
            .iter()
            .map(|(k, v)| format!("{k}{}  {v}", " ".repeat(width - k.chars().count())))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// No window could open (no display, or the windowing system refused): say it in text instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoWindow(pub String);

impl std::fmt::Display for NoWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no window could open: {}", self.0)
    }
}

impl std::error::Error for NoWindow {}

/// The windows, on the UI thread.
struct Windows {
    progress: ProgressWindow,
    page: PageWindow,
    pick: PickWindow,
    stage_progress: std::rc::Rc<slint::VecModel<f32>>,
    _shimmer_timer: slint::Timer,
    _tip_timer: slint::Timer,
}

thread_local! {
    static WINDOWS: std::cell::RefCell<Option<Windows>> = const { std::cell::RefCell::new(None) };
}

fn with<R>(f: impl FnOnce(&Windows) -> R) -> Option<R> {
    WINDOWS.with(|w| w.borrow().as_ref().map(f))
}

/// What Cancel in the progress window does.
type Cancel = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct State {
    /// The UI thread: started, or why it could not be.
    started: Option<Result<(), NoWindow>>,
    /// The build's first step (the progress window's clock).
    first_step: Option<Instant>,
    /// Start instant for elapsed clock display.
    progress_started: Option<Instant>,
    /// Whether the progress window is currently shown.
    progress_visible: bool,
    /// Recent lines displayed in the collapsible activity log drawer.
    log_lines: Vec<String>,
    /// The build is done: the progress window closed, not shown again.
    done: bool,
    /// The last line said in text (when there is no window).
    said: Option<(String, u32)>,
}

/// The builder's windows: a handle, cheap to clone, any thread's.
#[derive(Clone)]
pub struct Ui {
    state: Arc<Mutex<State>>,
    cancel: Arc<Mutex<Cancel>>,
}

impl Default for Ui {
    fn default() -> Ui {
        Ui::new()
    }
}

/// Cancel's default, as Ctrl-C stops a build: the builder ends at once (its children with it: build_core's child.rs),
/// exit status 130.
fn cancel_as_ctrl_c() {
    std::process::exit(130);
}

fn stage_badge_text(stage_index: i32) -> String {
    let raw_locale = build_core::system_locale().unwrap_or_else(|| "en-US".into());
    let family = raw_locale.split(['-', '_', '.', '@']).next().unwrap_or("en").to_ascii_lowercase();
    let (stage_word, of_word) = match family.as_str() {
        "pt" | "es" => ("ETAPA", "DE"),
        _ => ("STAGE", "OF"),
    };
    let n = (stage_index + 1).clamp(1, 6);
    format!("{stage_word} {n} {of_word} 6")
}

impl Ui {
    /// The windows, none open yet.
    pub fn new() -> Ui {
        Ui { state: Arc::default(), cancel: Arc::new(Mutex::new(Arc::new(cancel_as_ctrl_c))) }
    }

    /// The windows, the UI thread started now: Err when no window can open.
    pub fn open() -> Result<Ui, NoWindow> {
        let ui = Ui::new();
        ui.start()?;
        Ok(ui)
    }

    /// What Cancel in the progress window (or closing it) does, instead of ending the builder at once.
    pub fn on_cancel(&self, f: impl Fn() + Send + Sync + 'static) {
        *self.cancel.lock().unwrap_or_else(|e| e.into_inner()) = Arc::new(f);
    }

    fn start(&self) -> Result<(), NoWindow> {
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(r) = &st.started {
            return r.clone();
        }
        let (tx, rx) = mpsc::channel();
        let state = self.state.clone();
        let cancel = self.cancel.clone();
        let spawned = std::thread::Builder::new().name("ui".into()).spawn(move || ui_thread(tx, state, cancel));
        let r = match spawned {
            Ok(_) => rx.recv().unwrap_or_else(|_| Err(NoWindow("the window thread stopped".into()))),
            Err(e) => Err(NoWindow(e.to_string())),
        };
        st.started = Some(r.clone());
        r
    }

    /// Whether windows can open (the UI thread started, or starts now).
    pub fn available(&self) -> bool {
        self.start().is_ok()
    }

    /// The build's step and how far the whole build is (0 to 1).
    pub fn step(&self, text: &str, fraction: f64) {
        self.step_stage(-1, text, "", fraction, [0.0; 6]);
    }

    /// Updates the 6-stage progress stepper (`stage_index` in `0..=5`, or `< 0`), detail text, global fraction, and per-stage progress bars.
    pub fn step_stage(&self, stage_index: i32, text: &str, detail: &str, fraction: f64, stage_progress: [f32; 6]) {
        let first = {
            let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if st.done {
                return;
            }
            let first = st.first_step.is_none();
            st.first_step.get_or_insert_with(Instant::now);
            if first {
                st.progress_started = Some(Instant::now());
                if stage_index >= 0 {
                    st.progress_visible = true;
                }
            }
            first
        };
        if self.start().is_err() {
            self.say(text, fraction);
            return;
        }
        let (text, detail, fraction) = (text.to_string(), detail.to_string(), fraction.clamp(0.0, 1.0) as f32);
        let badge = (stage_index >= 0).then(|| stage_badge_text(stage_index));
        let state = self.state.clone();
        let _ = slint::invoke_from_event_loop(move || {
            with(|w| {
                w.progress.set_step(text.into());
                w.progress.set_detail(detail.into());
                w.progress.set_fraction(fraction);
                for (i, val) in stage_progress.iter().enumerate() {
                    w.stage_progress.set_row_data(i, *val);
                }
                if let Some(badge) = badge {
                    w.progress.set_active_stage(stage_index);
                    w.progress.set_stage_badge(badge.into());
                }
            });
            if first {
                if stage_index >= 0 {
                    with(|w| w.progress.show().ok());
                } else {
                    slint::Timer::single_shot(PROGRESS_AFTER, move || {
                        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                        if !st.done {
                            st.progress_visible = true;
                            drop(st);
                            with(|w| w.progress.show().ok());
                        }
                    });
                }
            }
        });
    }

    /// Appends a line to the collapsible live activity log drawer (retaining the last 80 lines).
    pub fn log_line(&self, line: &str) {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            return;
        }
        let text = {
            let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if !matches!(st.started, Some(Ok(()))) {
                return;
            }
            st.log_lines.push(trimmed.to_string());
            if st.log_lines.len() > 80 {
                let extra = st.log_lines.len() - 80;
                st.log_lines.drain(..extra);
            }
            st.log_lines.join("\n")
        };
        let _ = slint::invoke_from_event_loop(move || {
            with(|w| w.progress.set_log_text(text.into()));
        });
    }

    fn say(&self, text: &str, fraction: f64) {
        let pct = (fraction.clamp(0.0, 1.0) * 100.0).floor() as u32;
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let now = (text.to_string(), pct);
        if st.said.as_ref() != Some(&now) {
            eprintln!("xeonshot: {text} ({pct} %)");
            st.said = Some(now);
        }
    }

    /// The build is done (or the game starts): the progress window closes, and does not show again.
    pub fn done(&self) {
        let started = {
            let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
            st.done = true;
            st.progress_visible = false;
            matches!(st.started, Some(Ok(())))
        };
        if started {
            let _ = slint::invoke_from_event_loop(|| {
                with(|w| w.progress.hide().ok());
            });
        }
    }

    /// A page, until a button is pressed (its index) or the window is closed (None). The progress window hides while
    /// it shows.
    pub fn page(&self, page: &Page) -> Result<Option<usize>, NoWindow> {
        let last = page.buttons.len().saturating_sub(1);
        self.page_with_keys(page, 0, last)
    }

    /// [`Ui::page`] with the buttons Enter and Escape press (their indexes), a click being the other way to one.
    fn page_with_keys(&self, page: &Page, enter: usize, escape: usize) -> Result<Option<usize>, NoWindow> {
        self.start()?;
        let heading = {
            let t = page.title.strip_prefix("Xeonshot: ").or_else(|| page.title.strip_prefix("Longshot: ")).unwrap_or(page.title);
            let mut c = t.chars();
            c.next().map(|f| f.to_uppercase().chain(c).collect::<String>()).unwrap_or_default()
        };
        let (title, text) = (page.title.to_string(), page.text.to_string());
        let block = page.block.unwrap_or("").to_string();
        let buttons: Vec<slint::SharedString> = page.buttons.iter().map(|b| (*b).into()).collect();
        let n = buttons.len();
        let (tx, rx) = mpsc::channel::<Option<usize>>();
        let r = slint::invoke_from_event_loop(move || {
            with(|w| {
                let _ = w.progress.hide();
                let p = &w.page;
                p.set_window_title(title.into());
                p.set_heading(heading.into());
                p.set_text(text.into());
                p.set_block(block.into());
                p.set_buttons(std::rc::Rc::new(slint::VecModel::from(buttons)).into());
                let shown = Instant::now();
                let ready = move || shown.elapsed() >= PAGE_GUARD;
                let (t1, t2, t3) = (tx.clone(), tx.clone(), tx);
                let answered = std::rc::Rc::new(std::cell::Cell::new(false));
                let (a1, a2, a3) = (answered.clone(), answered.clone(), answered);
                let weak = p.as_weak();
                let finish = move |weak: &slint::Weak<PageWindow>,
                                   tx: &mpsc::Sender<Option<usize>>,
                                   a: &std::cell::Cell<bool>,
                                   hit| {
                    if !a.replace(true) {
                        let _ = tx.send(hit);
                        if let Some(p) = weak.upgrade() {
                            let _ = p.hide();
                        }
                    }
                };
                let (w1, w2) = (weak.clone(), weak);
                p.on_pressed(move |i| {
                    if ready() {
                        finish(&w1, &t1, &a1, Some(i as usize));
                    }
                });
                p.on_key(move |k, repeat| {
                    if !repeat && ready() && n > 0 {
                        finish(&w2, &t2, &a2, Some(if k == 0 { enter } else { escape }));
                    }
                });
                p.window().on_close_requested(move || {
                    if !a3.replace(true) {
                        let _ = t3.send(None);
                    }
                    slint::CloseRequestResponse::HideWindow
                });
                let _ = p.show();
            });
        });
        r.map_err(|e| NoWindow(e.to_string()))?;
        let hit = rx.recv().map_err(|_| NoWindow("the window thread stopped".into()))?;
        Ok(hit)
    }

    /// A question the player answers in a window of the builder's own: true only when the player pressed the accept
    /// button (a click: Enter, Escape, closing the window and a key in the first [`PAGE_GUARD`] all decline). The one
    /// way anything the game asks for, such as trusting a mod's code or granting a permission, is granted: the game only
    /// asks, it never answers.
    pub fn confirm(&self, c: &Confirm) -> Result<bool, NoWindow> {
        let text = c.lines.join("\n\n");
        let block = c.block();
        let hit = self.page_with_keys(
            &Page {
                title: c.title,
                text: &text,
                block: (!block.is_empty()).then_some(block.as_str()),
                buttons: &[c.accept, c.decline],
            },
            1,
            1,
        )?;
        Ok(hit == Some(0))
    }

    /// An error, until the player closes it (Close).
    pub fn error(&self, text: &str) -> Result<(), NoWindow> {
        let s = strings();
        self.page(&Page { title: s.error_title, text, block: None, buttons: &[s.close] }).map(|_| ())
    }

    /// The crash window and its pages ([`crash::flow::show`], with the desktop's clipboard, browser and curl): how it
    /// ended.
    pub fn crash_window(&self, shown: &crash::Shown) -> Result<Choice, NoWindow> {
        self.start()?;
        let mut pages = UiPages { ui: self, failed: None };
        let choice = crash::flow::show(shown, &mut pages, &mut Desk::system());
        match pages.failed {
            Some(e) => Err(e),
            None => Ok(choice),
        }
    }

    /// The first run's picker: the player's .nup (or a dump folder), by its path or the system's file chooser. None
    /// when the player cancelled.
    pub fn pick_game(&self) -> Result<Option<PathBuf>, NoWindow> {
        self.start()?;
        let s = strings();
        let missing_note = s.missing_file_note;
        let dialog_title = s.file_dialog_title;
        let (tx, rx) = mpsc::channel::<Option<PathBuf>>();
        let r = slint::invoke_from_event_loop(move || {
            with(|w| {
                let _ = w.progress.hide();
                let p = &w.pick;
                p.set_note("".into());
                let shown = Instant::now();
                let answered = std::rc::Rc::new(std::cell::Cell::new(false));
                let (a1, a2) = (answered.clone(), answered);
                let (t1, t2) = (tx.clone(), tx);
                let weak = p.as_weak();
                let w1 = weak.clone();
                p.on_done(move |ok| {
                    let Some(p) = w1.upgrade() else { return };
                    if shown.elapsed() < PAGE_GUARD || a1.get() {
                        return;
                    }
                    if !ok {
                        a1.set(true);
                        let _ = t1.send(None);
                        let _ = p.hide();
                        return;
                    }
                    let path = PathBuf::from(p.get_path().trim().trim_matches('"'));
                    if path.as_os_str().is_empty() || !path.exists() {
                        p.set_note(missing_note.into());
                        return;
                    }
                    a1.set(true);
                    let _ = t1.send(Some(path));
                    let _ = p.hide();
                });
                let w2 = weak;
                p.on_browse(move || {
                    let w = w2.clone();
                    std::thread::spawn(move || {
                        let file = rfd::FileDialog::new()
                            .set_title(dialog_title)
                            .add_filter("Ocarina of Time (Nintendo Switch 2)", &["nup"])
                            .pick_file();
                        if let Some(f) = file {
                            let _ = w.upgrade_in_event_loop(move |p| p.set_path(f.display().to_string().into()));
                        }
                    });
                });
                p.window().on_close_requested(move || {
                    if !a2.replace(true) {
                        let _ = t2.send(None);
                    }
                    slint::CloseRequestResponse::HideWindow
                });
                let _ = p.show();
            });
        });
        r.map_err(|e| NoWindow(e.to_string()))?;
        rx.recv().map_err(|_| NoWindow("the window thread stopped".into()))
    }
}

/// [`Pages`] on the builder's window; a window that can't open ends the flow (no button: Close).
pub struct UiPages<'a> {
    ui: &'a Ui,
    /// Why no window could open, when it could not.
    pub failed: Option<NoWindow>,
}

impl<'a> UiPages<'a> {
    /// Pages on `ui`.
    pub fn new(ui: &'a Ui) -> UiPages<'a> {
        UiPages { ui, failed: None }
    }
}

impl Pages for UiPages<'_> {
    fn page(&mut self, page: &Page) -> Option<usize> {
        if self.failed.is_some() {
            return None;
        }
        match self.ui.page(page) {
            Ok(hit) => hit,
            Err(e) => {
                self.failed = Some(e);
                None
            }
        }
    }
}

/// The UI thread: Slint's winit back end with its software renderer, the windows made (hidden), then its event loop
/// for as long as the builder runs (winit makes one event loop a process).
fn ui_thread(ready: mpsc::Sender<Result<(), NoWindow>>, state: Arc<Mutex<State>>, cancel: Arc<Mutex<Cancel>>) {
    let made = (|| -> Result<Windows, NoWindow> {
        let no = |e: slint::PlatformError| NoWindow(e.to_string());
        slint::BackendSelector::new()
            .backend_name("winit".into())
            .renderer_name("software".into())
            .select()
            .map_err(no)?;
        let s = strings();
        let stage_progress = std::rc::Rc::new(slint::VecModel::from(vec![0.0f32; 6]));
        let shimmer_timer = slint::Timer::default();
        let state_timer = state.clone();
        shimmer_timer.start(slint::TimerMode::Repeated, Duration::from_millis(60), move || {
            let (visible, elapsed) = match state_timer.try_lock() {
                Ok(st) => {
                    let secs = st.progress_started.map_or(0, |t| t.elapsed().as_secs());
                    (st.progress_visible, format!("{:02}:{:02}", secs / 60, secs % 60))
                }
                Err(_) => return,
            };
            if !visible {
                return;
            }
            with(|w| {
                let mut pos = w.progress.get_shimmer_pos() + 0.03;
                if pos > 1.0 {
                    pos = 0.0;
                }
                w.progress.set_shimmer_pos(pos);
                w.progress.set_elapsed(elapsed.into());
            });
        });
        let tip_timer = slint::Timer::default();
        let state_tips = state;
        let tip_count = s.tips.len() as i32;
        tip_timer.start(slint::TimerMode::Repeated, Duration::from_secs(9), move || {
            let Ok(st) = state_tips.try_lock() else { return };
            if !st.progress_visible || tip_count <= 0 {
                return;
            }
            drop(st);
            with(|w| {
                let next = (w.progress.get_tip_index() + 1) % tip_count;
                w.progress.set_tip_index(next);
            });
        });
        let w = Windows {
            progress: ProgressWindow::new().map_err(no)?,
            page: PageWindow::new().map_err(no)?,
            pick: PickWindow::new().map_err(no)?,
            stage_progress,
            _shimmer_timer: shimmer_timer,
            _tip_timer: tip_timer,
        };
        w.progress.set_window_title(s.window_title.into());
        w.progress.set_heading(s.progress_heading.into());
        w.progress.set_sub(s.progress_sub.into());
        w.progress.set_total_label(s.total_label.into());
        w.progress.set_cancel_label(s.cancel.into());
        let stage_labels: Vec<slint::SharedString> = s.stages.iter().map(|t| (*t).into()).collect();
        w.progress.set_stages(std::rc::Rc::new(slint::VecModel::from(stage_labels)).into());
        w.progress.set_stage_progress(w.stage_progress.clone().into());
        let stage_desc_labels: Vec<slint::SharedString> = s.stage_descs.iter().map(|t| (*t).into()).collect();
        w.progress.set_stage_descs(std::rc::Rc::new(slint::VecModel::from(stage_desc_labels)).into());
        w.progress.set_status_done(s.status_done.into());
        w.progress.set_status_active(s.status_active.into());
        w.progress.set_status_pending(s.status_pending.into());
        w.progress.set_follow_active_label(s.follow_active.into());
        w.progress.set_tip_badge(s.tip_badge.into());
        let tip_labels: Vec<slint::SharedString> = s.tips.iter().map(|t| (*t).into()).collect();
        w.progress.set_tips(std::rc::Rc::new(slint::VecModel::from(tip_labels)).into());
        w.progress.set_toggle_log_show(s.toggle_log_show.into());
        w.progress.set_toggle_log_hide(s.toggle_log_hide.into());
        w.progress.set_active_stage(0);
        w.progress.set_stage_badge(format!("{} 1 {} 6", s.stage_word, s.of_word).into());
        w.pick.set_window_title(s.installer_title.into());
        w.pick.set_heading(s.pick_heading.into());
        w.pick.set_sub(s.pick_sub.into());
        w.pick.set_placeholder(s.pick_placeholder.into());
        w.pick.set_browse_label(s.browse.into());
        w.pick.set_cancel_label(s.cancel.into());
        w.pick.set_continue_label(s.install_and_play.into());
        w.pick.set_card_lang_title(s.card_lang_title.into());
        w.pick.set_card_lang_value(s.lang_value.into());
        w.pick.set_card_display_title(s.card_display_title.into());
        w.pick.set_card_display_value(s.display_value.into());
        w.pick.set_card_preset_title(s.card_preset_title.into());
        w.pick.set_card_preset_value(s.preset_value.into());
        let c = cancel.clone();
        w.progress.on_cancel(move || {
            let f = c.lock().unwrap_or_else(|e| e.into_inner()).clone();
            f();
        });
        w.progress.window().on_close_requested(move || {
            let f = cancel.lock().unwrap_or_else(|e| e.into_inner()).clone();
            f();
            slint::CloseRequestResponse::HideWindow
        });
        Ok(w)
    })();
    match made {
        Ok(w) => {
            WINDOWS.with(|cell| *cell.borrow_mut() = Some(w));
            let _ = ready.send(Ok(()));
            let _ = slint::run_event_loop_until_quit();
        }
        Err(e) => {
            let _ = ready.send(Err(e));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_facts_are_one_a_line_with_their_values_aligned() {
        let c = Confirm {
            title: "Longshot: trust a mod's code?",
            lines: &["Trust this mod's code?"],
            facts: &[("Mod", "ann.fast-travel"), ("Source", "https://mods.example/repo"), ("Code hash", "9f2c")],
            accept: "Trust",
            decline: "Don't trust",
        };
        assert_eq!(c.block(), "Mod        ann.fast-travel\nSource     https://mods.example/repo\nCode hash  9f2c");
        assert_eq!(Confirm { facts: &[], ..c }.block(), "");
    }
}
