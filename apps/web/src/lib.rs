// SPDX-License-Identifier: MPL-2.0

//! Browser host for Raster Nights.

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::{cell::RefCell, io, rc::Rc, time::Duration};

    use raster_display::{
        DISPLAY_HEIGHT, DISPLAY_WIDTH, DisplayBuffer, GridPoint, copy_to_ratatui,
    };
    use raster_engine::{
        Application, CalendarDate, DeviceInput, FixedStepClock, HostKind, InputCapability,
        InputSystem, KeyCode, KeyModifiers, LiveRegion, PhysicalKey, PointerButton, SemanticId,
        SemanticNode, SemanticRole, SemanticUiTree,
    };
    use ratzilla::{
        CanvasBackend, WebEventHandler, WebGl2Backend, WebRenderer,
        backend::{canvas::CanvasBackendOptions, webgl2::WebGl2BackendOptions},
        event::{
            MouseButton as WebMouseButton, MouseEvent as WebMouseEvent,
            MouseEventKind as WebMouseEventKind,
        },
        ratatui::{Terminal, backend::Backend},
    };
    use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
    use web_sys::{Document, Element, Event, KeyboardEvent, Node};

    const DISPLAY_ELEMENT_ID: &str = "drx90-display";
    const SEMANTIC_ELEMENT_ID: &str = "drx90-semantic";
    // Ratzilla's bundled static atlas uses 12x20 cells.
    const WEBGL_PIXEL_SIZE: (u32, u32) = (1_200, 720);
    const CANVAS_PIXEL_SIZE: (u32, u32) = (1_000, 684);

    thread_local! {
        static EVENT_QUEUE: RefCell<Option<Rc<RefCell<Vec<BrowserEvent>>>>> =
            const { RefCell::new(None) };
    }

    enum BrowserEvent {
        Activity,
        Device(DeviceInput),
        SemanticActivate(SemanticId),
    }

    struct BrowserRuntime {
        app: Application,
        input: InputSystem,
        clock: FixedStepClock,
        display: DisplayBuffer,
        events: Rc<RefCell<Vec<BrowserEvent>>>,
        previous_frame_ms: f64,
        focus_suspended: bool,
        rendered_semantic_revision: Option<u64>,
    }

    impl BrowserRuntime {
        fn new(events: Rc<RefCell<Vec<BrowserEvent>>>) -> Self {
            Self {
                app: Application::new(HostKind::Browser, browser_date(), false),
                input: InputSystem::new(InputCapability::Enhanced),
                clock: FixedStepClock::new(),
                display: DisplayBuffer::canonical(),
                events,
                previous_frame_ms: js_sys::Date::now(),
                focus_suspended: false,
                rendered_semantic_revision: None,
            }
        }

        fn update(&mut self) {
            let now = js_sys::Date::now();
            let elapsed_ms = (now - self.previous_frame_ms).max(0.0);
            self.previous_frame_ms = now;

            let events = self.events.borrow_mut().drain(..).collect::<Vec<_>>();
            for event in events {
                self.handle_event(event);
            }

            let elapsed = Duration::from_secs_f64(elapsed_ms / 1_000.0);
            for step in self
                .clock
                .advance(elapsed, self.focus_suspended || self.app.is_suspended())
                .iter()
            {
                for action in self.input.advance(step.tick) {
                    self.app.handle_action(action.action, action.phase);
                }
                self.app.update(step);
            }

            if let Err(error) = self.app.render(&mut self.display) {
                self.app
                    .fail(format!("display composition failed: {error}"));
                let _ = self.app.render(&mut self.display);
            }

            let semantic_tree = self.app.semantic_tree();
            if self.rendered_semantic_revision != Some(semantic_tree.revision)
                && sync_semantic_tree(&semantic_tree).is_ok()
            {
                self.rendered_semantic_revision = Some(semantic_tree.revision);
            }
        }

        fn handle_event(&mut self, event: BrowserEvent) {
            match event {
                BrowserEvent::Activity => self.app.handle_activity(),
                BrowserEvent::Device(DeviceInput::FocusLost) => {
                    self.focus_suspended = true;
                    for action in self.input.release_all(self.clock.current_tick()) {
                        self.app.handle_action(action.action, action.phase);
                    }
                }
                BrowserEvent::Device(DeviceInput::FocusGained) => {
                    self.focus_suspended = false;
                }
                BrowserEvent::Device(DeviceInput::PointerPressed {
                    button: PointerButton::Primary,
                    column,
                    row,
                }) => self.app.handle_pointer_press(column, row),
                BrowserEvent::Device(device_input) => {
                    for action in self.input.handle(
                        device_input,
                        self.clock.current_tick(),
                        self.app.input_context(),
                    ) {
                        self.app.handle_action(action.action, action.phase);
                    }
                }
                BrowserEvent::SemanticActivate(id) => self.app.activate_semantic_node(&id),
            }
        }
    }

    /// Starts the DRX-90 browser renderer after explicit website power-on.
    #[wasm_bindgen]
    pub fn start() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();

        let events = Rc::new(RefCell::new(Vec::new()));
        EVENT_QUEUE.with(|queue| {
            *queue.borrow_mut() = Some(Rc::clone(&events));
        });
        register_browser_events(Rc::clone(&events))?;

        let runtime = Rc::new(RefCell::new(BrowserRuntime::new(events)));
        let webgl_options = WebGl2BackendOptions::new()
            .grid_id(DISPLAY_ELEMENT_ID)
            .size(WEBGL_PIXEL_SIZE)
            .disable_auto_css_resize();
        match WebGl2Backend::new_with_options(webgl_options) {
            Ok(backend) => run_renderer(backend, runtime),
            Err(_) => {
                clear_display_mount()?;
                let canvas_options = CanvasBackendOptions::new()
                    .grid_id(DISPLAY_ELEMENT_ID)
                    .size(CANVAS_PIXEL_SIZE);
                let backend = CanvasBackend::new_with_options(canvas_options).map_err(js_error)?;
                run_renderer(backend, runtime)
            }
        }
    }

    /// Activates one stable node from the browser accessibility mirror.
    #[wasm_bindgen]
    pub fn semantic_activate(id: String) {
        let Ok(id) = SemanticId::parse(id) else {
            return;
        };
        EVENT_QUEUE.with(|queue| {
            if let Some(events) = queue.borrow().as_ref() {
                events.borrow_mut().push(BrowserEvent::SemanticActivate(id));
            }
        });
    }

    fn run_renderer<B>(backend: B, runtime: Rc<RefCell<BrowserRuntime>>) -> Result<(), JsValue>
    where
        B: Backend<Error = io::Error> + WebEventHandler + 'static,
    {
        let mut terminal = Terminal::new(backend).map_err(js_error)?;
        let events = Rc::clone(&runtime.borrow().events);
        terminal
            .on_mouse_event(move |event| {
                if let Some(event) = map_mouse_event(event) {
                    events.borrow_mut().push(BrowserEvent::Device(event));
                }
            })
            .map_err(js_error)?;

        terminal.draw_web(move |frame| {
            let mut runtime = runtime.borrow_mut();
            runtime.update();
            frame.buffer_mut().reset();
            let area = frame.area();
            if area.width < DISPLAY_WIDTH || area.height < DISPLAY_HEIGHT {
                frame.buffer_mut().set_string(
                    area.x,
                    area.y,
                    "DRX-90 DISPLAY AREA UNAVAILABLE",
                    ratzilla::ratatui::style::Style::default()
                        .fg(ratzilla::ratatui::style::Color::Yellow),
                );
                return;
            }
            let origin = GridPoint::new(
                area.x + (area.width - DISPLAY_WIDTH) / 2,
                area.y + (area.height - DISPLAY_HEIGHT) / 2,
            );
            copy_to_ratatui(&runtime.display, frame.buffer_mut(), origin);
        });
        Ok(())
    }

    fn register_browser_events(events: Rc<RefCell<Vec<BrowserEvent>>>) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("document unavailable"))?;

        let keydown_events = Rc::clone(&events);
        let keydown = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event| {
            keydown_events.borrow_mut().push(BrowserEvent::Activity);
            let Some(key) = map_keyboard_event(&event) else {
                return;
            };
            if should_prevent_default(&key) {
                event.prevent_default();
            }
            let input = if event.repeat() {
                DeviceInput::KeyRepeated(key)
            } else {
                DeviceInput::KeyPressed(key)
            };
            keydown_events
                .borrow_mut()
                .push(BrowserEvent::Device(input));
        });
        document.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())?;
        keydown.forget();

        let keyup_events = Rc::clone(&events);
        let keyup = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event| {
            if let Some(key) = map_keyboard_event(&event) {
                keyup_events
                    .borrow_mut()
                    .push(BrowserEvent::Device(DeviceInput::KeyReleased(key)));
            }
        });
        document.add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())?;
        keyup.forget();

        let blur_events = Rc::clone(&events);
        let blur = Closure::<dyn FnMut(Event)>::new(move |_| {
            blur_events
                .borrow_mut()
                .push(BrowserEvent::Device(DeviceInput::FocusLost));
        });
        window.add_event_listener_with_callback("blur", blur.as_ref().unchecked_ref())?;
        blur.forget();

        let focus_events = Rc::clone(&events);
        let focus = Closure::<dyn FnMut(Event)>::new(move |_| {
            focus_events
                .borrow_mut()
                .push(BrowserEvent::Device(DeviceInput::FocusGained));
        });
        window.add_event_listener_with_callback("focus", focus.as_ref().unchecked_ref())?;
        focus.forget();

        let visibility_events = events;
        let visibility_document = document.clone();
        let visibility = Closure::<dyn FnMut(Event)>::new(move |_| {
            let input = if visibility_document.hidden() {
                DeviceInput::FocusLost
            } else {
                DeviceInput::FocusGained
            };
            visibility_events
                .borrow_mut()
                .push(BrowserEvent::Device(input));
        });
        document.add_event_listener_with_callback(
            "visibilitychange",
            visibility.as_ref().unchecked_ref(),
        )?;
        visibility.forget();

        Ok(())
    }

    fn map_keyboard_event(event: &KeyboardEvent) -> Option<PhysicalKey> {
        let code = match event.key().as_str() {
            "ArrowLeft" => KeyCode::ArrowLeft,
            "ArrowRight" => KeyCode::ArrowRight,
            "ArrowUp" => KeyCode::ArrowUp,
            "ArrowDown" => KeyCode::ArrowDown,
            "Enter" => KeyCode::Enter,
            "Escape" => KeyCode::Escape,
            " " => KeyCode::Space,
            "Tab" => KeyCode::Tab,
            "Backspace" => KeyCode::Backspace,
            "Delete" => KeyCode::Delete,
            "Home" => KeyCode::Home,
            "End" => KeyCode::End,
            value if value.starts_with('F') => {
                let number = value[1..].parse().ok()?;
                KeyCode::Function(number)
            }
            value => {
                let mut characters = value.chars();
                let character = characters.next()?;
                if characters.next().is_some() {
                    return None;
                }
                KeyCode::Character(character)
            }
        };
        Some(PhysicalKey {
            code,
            modifiers: KeyModifiers {
                control: event.ctrl_key(),
                alt: event.alt_key(),
                shift: event.shift_key(),
            },
        })
    }

    fn should_prevent_default(key: &PhysicalKey) -> bool {
        matches!(
            key.code,
            KeyCode::ArrowLeft
                | KeyCode::ArrowRight
                | KeyCode::ArrowUp
                | KeyCode::ArrowDown
                | KeyCode::Enter
                | KeyCode::Escape
                | KeyCode::Space
        ) || (key.modifiers.control && matches!(key.code, KeyCode::Character('c' | 'C')))
    }

    fn map_mouse_event(event: WebMouseEvent) -> Option<DeviceInput> {
        let button = match event.kind {
            WebMouseEventKind::ButtonDown(button)
            | WebMouseEventKind::ButtonUp(button)
            | WebMouseEventKind::SingleClick(button)
            | WebMouseEventKind::DoubleClick(button) => map_mouse_button(button)?,
            WebMouseEventKind::Moved => {
                return Some(DeviceInput::PointerMoved {
                    column: event.col,
                    row: event.row,
                });
            }
            _ => return None,
        };
        match event.kind {
            WebMouseEventKind::ButtonDown(_) | WebMouseEventKind::SingleClick(_) => {
                Some(DeviceInput::PointerPressed {
                    button,
                    column: event.col,
                    row: event.row,
                })
            }
            WebMouseEventKind::ButtonUp(_) => Some(DeviceInput::PointerReleased {
                button,
                column: event.col,
                row: event.row,
            }),
            _ => None,
        }
    }

    fn map_mouse_button(button: WebMouseButton) -> Option<PointerButton> {
        match button {
            WebMouseButton::Left => Some(PointerButton::Primary),
            WebMouseButton::Right => Some(PointerButton::Secondary),
            WebMouseButton::Middle => Some(PointerButton::Middle),
            _ => None,
        }
    }

    fn sync_semantic_tree(tree: &SemanticUiTree) -> Result<(), JsValue> {
        let document = browser_document()?;
        let mount = document
            .get_element_by_id(SEMANTIC_ELEMENT_ID)
            .ok_or_else(|| JsValue::from_str("semantic UI mount unavailable"))?;
        mount.set_inner_html("");
        let root: Node = semantic_element(&document, &tree.root)?.into();
        mount.append_child(&root)?;
        Ok(())
    }

    fn semantic_element(document: &Document, node: &SemanticNode) -> Result<Element, JsValue> {
        let tag = if node.role == SemanticRole::Button {
            "button"
        } else {
            "div"
        };
        let element = document.create_element(tag)?;
        element.set_attribute("data-semantic-id", node.id.as_str())?;
        element.set_attribute("role", semantic_role(node.role))?;
        element.set_attribute("aria-label", &node.label)?;
        if node.state.focused {
            element.set_attribute("aria-current", "true")?;
        }
        if node.state.disabled {
            element.set_attribute("disabled", "")?;
        }
        if let Some(live) = node.state.live {
            element.set_attribute(
                "aria-live",
                match live {
                    LiveRegion::Polite => "polite",
                    LiveRegion::Assertive => "assertive",
                },
            )?;
        }
        element.set_text_content(Some(&node.label));
        for child in &node.children {
            let child: Node = semantic_element(document, child)?.into();
            element.append_child(&child)?;
        }
        Ok(element)
    }

    const fn semantic_role(role: SemanticRole) -> &'static str {
        match role {
            SemanticRole::Application => "application",
            SemanticRole::Dialog => "dialog",
            SemanticRole::Heading => "heading",
            SemanticRole::List => "list",
            SemanticRole::ListItem => "listitem",
            SemanticRole::Button => "button",
            SemanticRole::Status => "status",
            SemanticRole::TextInput => "textbox",
            SemanticRole::Grid => "grid",
            SemanticRole::Row => "row",
            SemanticRole::GridCell => "gridcell",
        }
    }

    fn browser_date() -> CalendarDate {
        let date = js_sys::Date::new_0();
        CalendarDate::new(
            date.get_date() as u8,
            date.get_month().saturating_add(1) as u8,
            date.get_full_year() as i32,
        )
    }

    fn browser_document() -> Result<Document, JsValue> {
        web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("document unavailable"))
    }

    fn clear_display_mount() -> Result<(), JsValue> {
        let mount = browser_document()?
            .get_element_by_id(DISPLAY_ELEMENT_ID)
            .ok_or_else(|| JsValue::from_str("DRX-90 display mount unavailable"))?;
        mount.set_inner_html("");
        Ok(())
    }

    fn js_error(error: impl std::fmt::Display) -> JsValue {
        JsValue::from_str(&error.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{semantic_activate, start};
