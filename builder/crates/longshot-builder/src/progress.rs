//! What a person sees of a build with `--output user` (output.rs): one small seam, [`UserUi`], that the build's steps,
//! its errors and its end go through. Its one implementation for now, [`Stderr`], says the steps on stderr and an error
//! in the system's message box (play.rs's `say`); the builder's own windows (Slint, with the crash window) come behind
//! the same trait.

use build_core::report::{Event, Stage};

/// A person's view of a build: its step and how far it is, an error, its end.
pub trait UserUi: Send {
    /// The step's sentence and how far the whole build is (0 to 1).
    fn step(&mut self, text: &str, fraction: f64);
    /// The build's current stepper stage (`0..=5`, or `< 0` if unknown), step text, live detail line, and fraction.
    fn stage_step(&mut self, _stage_index: i32, text: &str, _detail: &str, fraction: f64) {
        self.step(text, fraction);
    }
    /// The build stopped: why, for the player.
    fn error(&mut self, text: &str);
    /// The build is done.
    fn done(&mut self);
}

/// Live stage, module compilation count, and ETA tracker for the Xeonshot installer stepper.
#[derive(Default)]
pub struct StageTracker {
    /// Current 0-based stage index (`0..=5`, or `-1` before the first stage).
    pub stage_index: i32,
    /// Current human-readable stage text.
    pub step: String,
    /// Current progress fraction (`0.0..=1.0`).
    pub fraction: f64,
    units_done: usize,
    units_total: usize,
    eta_secs: Option<u64>,
}

impl StageTracker {
    /// Applies a build event and returns `true` if the visible UI state changed.
    pub fn apply(&mut self, event: &Event) -> bool {
        match event {
            Event::Stage { stage, text } => {
                self.stage_index = match stage {
                    Stage::Tools
                    | Stage::Source
                    | Stage::Rust
                    | Stage::SourceTools
                    | Stage::Crates
                    | Stage::Pack
                    | Stage::Builder => 0,
                    Stage::Read => 1,
                    Stage::Lift => 2,
                    Stage::Compile => 3,
                    Stage::Finish => 4,
                    Stage::Shaders => 5,
                };
                self.step = text.clone();
                if *stage != Stage::Compile {
                    self.units_done = 0;
                    self.units_total = 0;
                    self.eta_secs = None;
                }
                true
            }
            Event::Progress { stage: Stage::Shaders, fraction, .. } => {
                self.stage_index = 5;
                self.fraction = fraction.clamp(0.0, 1.0);
                true
            }
            Event::Progress { overall, .. } => {
                self.fraction = overall.clamp(0.0, 1.0);
                true
            }
            Event::CompilePlan { total, .. } => {
                self.units_done = 0;
                self.units_total = *total;
                true
            }
            Event::Unit { .. } => {
                self.units_done += 1;
                true
            }
            Event::Eta { seconds } => {
                self.eta_secs = Some(*seconds);
                true
            }
            _ => false,
        }
    }

    /// Formats a localized subtitle line with module count and ETA when available.
    pub fn detail(&self) -> String {
        let lang = build_core::system_locale()
            .unwrap_or_default()
            .split(['-', '_', '.', '@'])
            .next()
            .unwrap_or("en")
            .to_ascii_lowercase();
        let mut parts = Vec::new();
        if self.stage_index == 3 && self.units_total > 0 {
            let mods = match lang.as_str() {
                "pt" => format!("Módulos: {} / {}", self.units_done, self.units_total),
                "es" => format!("Módulos: {} / {}", self.units_done, self.units_total),
                _ => format!("Modules: {} / {}", self.units_done, self.units_total),
            };
            parts.push(mods);
        }
        if let Some(secs) = self.eta_secs {
            let mins = secs / 60;
            let rem = secs % 60;
            let eta = match lang.as_str() {
                "pt" if mins > 0 => format!("Tempo restante: ~{mins}m {rem:02}s"),
                "pt" => format!("Tempo restante: ~{rem}s"),
                "es" if mins > 0 => format!("Tiempo restante: ~{mins}m {rem:02}s"),
                "es" => format!("Tiempo restante: ~{rem}s"),
                _ if mins > 0 => format!("ETA: ~{mins}m {rem:02}s"),
                _ => format!("ETA: ~{rem}s"),
            };
            parts.push(eta);
        }
        if !parts.is_empty() {
            return parts.join("  •  ");
        }
        if lang == "pt" {
            return match self.stage_index {
                0 => "Preparando ferramentas, SDK e ambiente local...".into(),
                1 => "Lendo arquivos da ROM (.nup) e validando hashes...".into(),
                2 => "Traduzindo binário de Switch 2 para LLVM IR...".into(),
                3 => "Compilando e linkando binário nativo x86-64...".into(),
                4 => "Aplicando idioma, resolução do monitor e preset Xeonshot...".into(),
                5 => "Compilando shaders Vulkan para evitar engasgos no jogo...".into(),
                _ => self.step.clone(),
            };
        }
        if lang == "es" {
            return match self.stage_index {
                0 => "Preparando herramientas y entorno local...".into(),
                1 => "Leyendo archivos de la ROM (.nup)...".into(),
                2 => "Traduciendo código del juego a LLVM IR...".into(),
                3 => "Compilando ejecutable nativo x86-64...".into(),
                4 => "Aplicando idioma, resolución y preset Xeonshot...".into(),
                5 => "Compilando shaders Vulkan...".into(),
                _ => self.step.clone(),
            };
        }
        self.step.clone()
    }
}

/// A build event as a person's view shows it: the step's sentence (`step`) and how far (`fraction`: the whole
/// build's; the shader preparation, after the build's own steps, its own). Whether the event changed either.
pub fn shown(event: &Event, step: &mut String, fraction: &mut f64) -> bool {
    match event {
        Event::Stage { text, .. } => *step = text.clone(),
        Event::Progress { stage: Stage::Shaders, fraction: f, .. } => *fraction = f.clamp(0.0, 1.0),
        Event::Progress { overall, .. } => *fraction = overall.clamp(0.0, 1.0),
        _ => return false,
    }
    true
}

/// [`UserUi`] on stderr: a line each time the step or the whole percent changes; an error in a message box.
#[derive(Default)]
pub struct Stderr {
    last: Option<(String, u32)>,
}

impl Stderr {
    /// The line said for a step and a fraction, when it differs from the last one said.
    fn line(&mut self, text: &str, fraction: f64) -> Option<String> {
        let pct = (fraction.clamp(0.0, 1.0) * 100.0).floor() as u32;
        let now = (text.to_string(), pct);
        if self.last.as_ref() == Some(&now) {
            return None;
        }
        self.last = Some(now);
        Some(format!("longshot: {text} ({pct} %)"))
    }
}

impl UserUi for Stderr {
    fn step(&mut self, text: &str, fraction: f64) {
        if let Some(line) = self.line(text, fraction) {
            eprintln!("{line}");
        }
    }

    fn error(&mut self, text: &str) {
        crate::play::say(text, true, true);
    }

    fn done(&mut self) {}
}

/// The person's view for this run.
pub fn open() -> Box<dyn UserUi> {
    if crate::output::chosen().windows() {
        return Box::new(crate::after::Windows);
    }
    Box::new(Stderr::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_person_sees_the_step_and_the_whole_build_s_fraction() {
        let (mut step, mut f) = (String::new(), 0.0);
        let stage = Event::Stage { stage: Stage::Compile, text: "Compiling the game".into() };
        assert!(shown(&stage, &mut step, &mut f));
        assert_eq!(step, "Compiling the game");
        assert!(shown(&Event::Progress { stage: Stage::Compile, fraction: 0.5, overall: 0.565 }, &mut step, &mut f));
        assert_eq!(f, 0.565);
        // (the shaders: their own fraction, the build's overall held at 1)
        assert!(shown(&Event::Progress { stage: Stage::Shaders, fraction: 0.25, overall: 1.0 }, &mut step, &mut f));
        assert_eq!(f, 0.25);
        assert!(!shown(&Event::Log { text: "x".into() }, &mut step, &mut f));
    }

    #[test]
    fn stderr_says_each_change_once() {
        let mut s = Stderr::default();
        assert_eq!(s.line("Compiling the game", 0.561).as_deref(), Some("longshot: Compiling the game (56 %)"));
        assert_eq!(s.line("Compiling the game", 0.569), None, "the same percent");
        assert_eq!(s.line("Compiling the game", 0.575).as_deref(), Some("longshot: Compiling the game (57 %)"));
        assert_eq!(s.line("Preparing the shaders", 0.575).as_deref(), Some("longshot: Preparing the shaders (57 %)"));
    }
}
