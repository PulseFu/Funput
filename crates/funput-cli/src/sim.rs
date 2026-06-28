//! Platform simulation: feed a string through the engine and reconstruct the
//! app text the user would see, plus per-keystroke detail.
//!
//! This is exactly the work a real platform shell does — apply each
//! [`ImeResult`](funput_engine::ImeResult) to the app's text — so the CLI acts
//! as a minimal, scriptable "platform" with no input hooks.

use funput_core::{InputMethod, ToneStyle};
use funput_engine::{Action, Engine};

/// Input method selectable on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Telex,
    Vni,
}

impl Method {
    fn to_core(self) -> InputMethod {
        match self {
            Method::Telex => InputMethod::Telex,
            Method::Vni => InputMethod::Vni,
        }
    }
}

/// One keystroke and what the engine asked the platform to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub key: char,
    pub action: Action,
    pub backspace: usize,
    pub output: String,
    /// Engine composition buffer after this keystroke.
    pub buffer: String,
}

/// Result of running a whole input string through the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Simulation {
    /// Text the user would see in the app after typing `input`.
    pub app_text: String,
    pub steps: Vec<Step>,
}

/// Run `input` through a fresh engine, acting as the platform: apply each
/// `ImeResult` to an app-text model exactly like a real shell would.
/// Engine configuration for a simulation run. Mirrors the toggles a platform shell
/// would push to the engine.
#[derive(Debug, Clone, Copy)]
pub struct SimConfig {
    pub method: Method,
    pub tone_style: ToneStyle,
    pub smart_restore: bool,
    pub spell_check: bool,
}

impl SimConfig {
    /// Defaults matching a fresh engine (smart restore on, spell-check off).
    pub fn new(method: Method) -> Self {
        Self {
            method,
            tone_style: ToneStyle::Traditional,
            smart_restore: true,
            spell_check: false,
        }
    }
}

pub fn simulate(method: Method, input: &str) -> Simulation {
    simulate_with(SimConfig::new(method), input)
}

/// Like [`simulate`], but with explicit engine configuration.
pub fn simulate_with(config: SimConfig, input: &str) -> Simulation {
    let mut engine = Engine::new();
    engine.set_method(config.method.to_core());
    engine.set_tone_style(config.tone_style);
    engine.set_smart_restore(config.smart_restore);
    engine.set_spell_check(config.spell_check);

    let mut app_text = String::new();
    let mut steps = Vec::new();

    for key in input.chars() {
        let result = engine.process_char(key);
        match result.action {
            Action::None => app_text.push(key),
            // `Send` and (future) `Restore` both delete then inject.
            Action::Send | Action::Restore => {
                for _ in 0..result.backspace {
                    app_text.pop();
                }
                app_text.push_str(&result.output);
            }
        }
        steps.push(Step {
            key,
            action: result.action,
            backspace: result.backspace,
            output: result.output,
            buffer: engine.buffer().to_owned(),
        });
    }

    Simulation { app_text, steps }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(method: Method, input: &str) -> String {
        simulate(method, input).app_text
    }

    #[test]
    fn telex_basic_and_words() {
        assert_eq!(app(Method::Telex, "as"), "á");
        assert_eq!(app(Method::Telex, "dd"), "đ");
        assert_eq!(app(Method::Telex, "xins chaof"), "xín chào");
        assert_eq!(app(Method::Telex, "truowng"), "trương");
    }

    #[test]
    fn vni_basic() {
        assert_eq!(app(Method::Vni, "a1"), "á");
        assert_eq!(app(Method::Vni, "d9"), "đ");
        assert_eq!(app(Method::Vni, "ma1 ca2"), "má cà");
    }

    #[test]
    fn english_restore_on_boundary() {
        assert_eq!(app(Method::Telex, "card "), "card ");
        assert_eq!(app(Method::Telex, "cool "), "cool ");
        // A valid syllable is intentional — kept.
        assert_eq!(app(Method::Telex, "mas "), "má ");
    }

    #[test]
    fn steps_record_each_keystroke() {
        let sim = simulate(Method::Telex, "as");
        assert_eq!(sim.steps.len(), 2);

        assert_eq!(sim.steps[0].action, Action::None);
        assert_eq!(sim.steps[0].buffer, "a");

        assert_eq!(sim.steps[1].action, Action::Send);
        assert_eq!(sim.steps[1].backspace, 1);
        assert_eq!(sim.steps[1].output, "á");
        assert_eq!(sim.steps[1].buffer, "á");
    }
}
