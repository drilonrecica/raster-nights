// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;

use crate::{
    AppAction, DeviceInput, GameAction, InputCapability, InputContext, KeyCode, PhysicalKey,
    SimulationTick, TextEscapeBehavior,
};

const COMPATIBILITY_ARM_WINDOW: u64 = 60;
const COMPATIBILITY_LEASE_TICKS: u64 = 12;

/// Phase of an engine-owned semantic action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionPhase {
    Pressed,
    Repeated,
    Released,
}

/// Semantic action tagged with its authoritative tick.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionEvent {
    pub tick: SimulationTick,
    pub action: AppAction,
    pub phase: ActionPhase,
}

/// Engine-controlled repeat timing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepeatProfile {
    pub delay_ticks: u64,
    pub interval_ticks: u64,
}

impl RepeatProfile {
    #[must_use]
    pub const fn for_action(action: AppAction) -> Option<Self> {
        match action {
            AppAction::NavigateLeft
            | AppAction::NavigateRight
            | AppAction::NavigateUp
            | AppAction::NavigateDown => Some(Self {
                delay_ticks: 15,
                interval_ticks: 4,
            }),
            AppAction::Game(GameAction::MoveLeft | GameAction::MoveRight) => Some(Self {
                delay_ticks: 10,
                interval_ticks: 2,
            }),
            AppAction::Game(GameAction::MoveUp | GameAction::MoveDown | GameAction::SoftDrop) => {
                Some(Self {
                    delay_ticks: 10,
                    interval_ticks: 1,
                })
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HeldInput {
    action: AppAction,
    pressed_tick: SimulationTick,
    last_repeat_tick: SimulationTick,
    lease_expires: Option<SimulationTick>,
}

/// Shared held-key state and deterministic semantic repeat generator.
#[derive(Debug)]
pub struct InputSystem {
    capability: InputCapability,
    held: BTreeMap<PhysicalKey, HeldInput>,
    compatibility_candidates: BTreeMap<PhysicalKey, SimulationTick>,
}

impl InputSystem {
    #[must_use]
    pub fn new(capability: InputCapability) -> Self {
        Self {
            capability,
            held: BTreeMap::new(),
            compatibility_candidates: BTreeMap::new(),
        }
    }

    #[must_use]
    pub const fn capability(&self) -> InputCapability {
        self.capability
    }

    #[must_use]
    pub fn is_held(&self, key: PhysicalKey) -> bool {
        self.held.contains_key(&key)
    }

    /// Consumes one host key transition.
    pub fn handle(
        &mut self,
        input: DeviceInput,
        tick: SimulationTick,
        context: InputContext,
    ) -> Vec<ActionEvent> {
        match input {
            DeviceInput::KeyPressed(key) | DeviceInput::KeyRepeated(key) => {
                self.handle_press(key, tick, context)
            }
            DeviceInput::KeyReleased(key) => self.handle_release(key, tick),
            _ => Vec::new(),
        }
    }

    /// Advances held-key leases and emits engine-timed repeats.
    pub fn advance(&mut self, tick: SimulationTick) -> Vec<ActionEvent> {
        self.compatibility_candidates.retain(|_, candidate_tick| {
            tick.0.saturating_sub(candidate_tick.0) <= COMPATIBILITY_ARM_WINDOW
        });

        let mut events = Vec::new();
        self.held.retain(|_, held| {
            if held.lease_expires.is_some_and(|expiry| tick.0 >= expiry.0) {
                events.push(ActionEvent {
                    tick,
                    action: held.action,
                    phase: ActionPhase::Released,
                });
                return false;
            }

            let Some(profile) = RepeatProfile::for_action(held.action) else {
                return true;
            };
            let held_for = tick.0.saturating_sub(held.pressed_tick.0);
            let since_repeat = tick.0.saturating_sub(held.last_repeat_tick.0);
            if held_for >= profile.delay_ticks && since_repeat >= profile.interval_ticks {
                events.push(ActionEvent {
                    tick,
                    action: held.action,
                    phase: ActionPhase::Repeated,
                });
                held.last_repeat_tick = tick;
            }
            true
        });
        events
    }

    /// Releases all logical keys, for example when a host loses focus.
    pub fn release_all(&mut self, tick: SimulationTick) -> Vec<ActionEvent> {
        self.compatibility_candidates.clear();
        let held = std::mem::take(&mut self.held);
        held.into_values()
            .map(|held| ActionEvent {
                tick,
                action: held.action,
                phase: ActionPhase::Released,
            })
            .collect()
    }

    fn handle_press(
        &mut self,
        key: PhysicalKey,
        tick: SimulationTick,
        context: InputContext,
    ) -> Vec<ActionEvent> {
        let Some(action) = map_key_to_action(key, context) else {
            return Vec::new();
        };

        match self.capability {
            InputCapability::Enhanced => {
                if self.held.contains_key(&key) {
                    return Vec::new();
                }
                self.held.insert(key, HeldInput::new(action, tick, None));
                vec![ActionEvent {
                    tick,
                    action,
                    phase: ActionPhase::Pressed,
                }]
            }
            InputCapability::Compatibility => {
                if let Some(held) = self.held.get_mut(&key) {
                    held.lease_expires = Some(SimulationTick(
                        tick.0.saturating_add(COMPATIBILITY_LEASE_TICKS),
                    ));
                    return Vec::new();
                }

                if let Some(first_tick) = self.compatibility_candidates.get(&key).copied()
                    && tick.0.saturating_sub(first_tick.0) <= COMPATIBILITY_ARM_WINDOW
                {
                    self.compatibility_candidates.remove(&key);
                    self.held.insert(
                        key,
                        HeldInput::new(
                            action,
                            first_tick,
                            Some(SimulationTick(
                                tick.0.saturating_add(COMPATIBILITY_LEASE_TICKS),
                            )),
                        ),
                    );
                    return Vec::new();
                }

                self.compatibility_candidates.insert(key, tick);
                vec![ActionEvent {
                    tick,
                    action,
                    phase: ActionPhase::Pressed,
                }]
            }
        }
    }

    fn handle_release(&mut self, key: PhysicalKey, tick: SimulationTick) -> Vec<ActionEvent> {
        if self.capability != InputCapability::Enhanced {
            return Vec::new();
        }
        self.held
            .remove(&key)
            .map(|held| {
                vec![ActionEvent {
                    tick,
                    action: held.action,
                    phase: ActionPhase::Released,
                }]
            })
            .unwrap_or_default()
    }
}

impl HeldInput {
    const fn new(
        action: AppAction,
        pressed_tick: SimulationTick,
        lease_expires: Option<SimulationTick>,
    ) -> Self {
        Self {
            action,
            pressed_tick,
            last_repeat_tick: pressed_tick,
            lease_expires,
        }
    }
}

/// Maps a host-neutral key to a semantic action for the active context.
#[must_use]
pub fn map_key_to_action(key: PhysicalKey, context: InputContext) -> Option<AppAction> {
    if key.modifiers.control && matches!(key.code, KeyCode::Character('c' | 'C')) {
        return Some(AppAction::Interrupt);
    }

    match context {
        InputContext::Navigation => match key.code {
            KeyCode::ArrowLeft | KeyCode::Character('h' | 'H') => Some(AppAction::NavigateLeft),
            KeyCode::ArrowRight | KeyCode::Character('l' | 'L') => Some(AppAction::NavigateRight),
            KeyCode::ArrowUp | KeyCode::Character('k' | 'K') => Some(AppAction::NavigateUp),
            KeyCode::ArrowDown | KeyCode::Character('j' | 'J') => Some(AppAction::NavigateDown),
            KeyCode::Enter | KeyCode::Space => Some(AppAction::Confirm),
            KeyCode::Escape => Some(AppAction::Back),
            _ => None,
        },
        InputContext::Gameplay => match key.code {
            KeyCode::ArrowLeft => Some(AppAction::Game(GameAction::MoveLeft)),
            KeyCode::ArrowRight => Some(AppAction::Game(GameAction::MoveRight)),
            KeyCode::ArrowUp | KeyCode::Character('x' | 'X') => {
                Some(AppAction::Game(GameAction::RotateClockwise))
            }
            KeyCode::Character('z' | 'Z') => {
                Some(AppAction::Game(GameAction::RotateCounterclockwise))
            }
            KeyCode::ArrowDown => Some(AppAction::Game(GameAction::SoftDrop)),
            KeyCode::Space => Some(AppAction::Game(GameAction::HardDrop)),
            KeyCode::Character('c' | 'C') => Some(AppAction::Game(GameAction::Hold)),
            KeyCode::Escape => Some(AppAction::Pause),
            _ => None,
        },
        InputContext::TextEntry(escape_behavior) => match key.code {
            KeyCode::Escape => Some(match escape_behavior {
                TextEscapeBehavior::Clear => AppAction::ClearText,
                TextEscapeBehavior::Back => AppAction::Back,
            }),
            KeyCode::Backspace => Some(AppAction::DeleteBackward),
            KeyCode::Delete => Some(AppAction::DeleteForward),
            KeyCode::Enter => Some(AppAction::Confirm),
            KeyCode::Character(character) if !key.modifiers.control && !key.modifiers.alt => {
                Some(AppAction::TextInput(character))
            }
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KeyModifiers;

    const LEFT: PhysicalKey = PhysicalKey::new(KeyCode::ArrowLeft);

    #[test]
    fn text_context_does_not_treat_hjkl_as_navigation() {
        let action = map_key_to_action(
            PhysicalKey::new(KeyCode::Character('h')),
            InputContext::TextEntry(TextEscapeBehavior::Clear),
        );

        assert_eq!(action, Some(AppAction::TextInput('h')));
    }

    #[test]
    fn control_c_is_always_interrupt() {
        let action = map_key_to_action(
            PhysicalKey {
                code: KeyCode::Character('c'),
                modifiers: KeyModifiers {
                    control: true,
                    ..KeyModifiers::default()
                },
            },
            InputContext::Gameplay,
        );

        assert_eq!(action, Some(AppAction::Interrupt));
    }

    #[test]
    fn enhanced_mode_ignores_raw_repeat_and_uses_engine_repeat() {
        let mut input = InputSystem::new(InputCapability::Enhanced);
        let pressed = input.handle(
            DeviceInput::KeyPressed(LEFT),
            SimulationTick(1),
            InputContext::Navigation,
        );
        let raw_repeat = input.handle(
            DeviceInput::KeyRepeated(LEFT),
            SimulationTick(10),
            InputContext::Navigation,
        );

        assert_eq!(pressed[0].phase, ActionPhase::Pressed);
        assert!(raw_repeat.is_empty());
        assert!(input.advance(SimulationTick(15)).is_empty());
        assert_eq!(
            input.advance(SimulationTick(16))[0].phase,
            ActionPhase::Repeated
        );
        assert_eq!(
            input.handle(
                DeviceInput::KeyReleased(LEFT),
                SimulationTick(17),
                InputContext::Navigation,
            )[0]
            .phase,
            ActionPhase::Released
        );
    }

    #[test]
    fn compatibility_mode_arms_refreshes_and_expires_lease() {
        let mut input = InputSystem::new(InputCapability::Compatibility);
        assert_eq!(
            input.handle(
                DeviceInput::KeyPressed(LEFT),
                SimulationTick(1),
                InputContext::Navigation,
            )[0]
            .phase,
            ActionPhase::Pressed
        );
        assert!(
            input
                .handle(
                    DeviceInput::KeyRepeated(LEFT),
                    SimulationTick(10),
                    InputContext::Navigation,
                )
                .is_empty()
        );
        assert!(input.is_held(LEFT));

        input.handle(
            DeviceInput::KeyRepeated(LEFT),
            SimulationTick(20),
            InputContext::Navigation,
        );
        assert_eq!(
            input.advance(SimulationTick(31))[0].phase,
            ActionPhase::Repeated
        );
        assert_eq!(
            input.advance(SimulationTick(32))[0].phase,
            ActionPhase::Released
        );
        assert!(!input.is_held(LEFT));
    }

    #[test]
    fn focus_cleanup_releases_all_held_keys() {
        let mut input = InputSystem::new(InputCapability::Enhanced);
        input.handle(
            DeviceInput::KeyPressed(LEFT),
            SimulationTick(1),
            InputContext::Navigation,
        );

        let released = input.release_all(SimulationTick(2));

        assert_eq!(released[0].phase, ActionPhase::Released);
        assert!(!input.is_held(LEFT));
    }
}
