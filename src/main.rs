use hoo_engine::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use iced::widget::{column, container, pick_list, scrollable, vertical_space};
use iced::{Alignment, Element, Length, Sandbox, Settings};


pub fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let hoo_engine = HooEngine::new_async(&window);
    let hoo_engine = futures::executor::block_on(hoo_engine);

    initialize_hoo_engine(hoo_engine.clone());

    hoo_engine.borrow().prepare();

    {
        let hoo_engine_ref = hoo_engine.borrow();
        let mut renderer_ref = hoo_engine_ref.get_renderer_mut();
        let init_future = renderer_ref.initialize_test_resources();
        futures::executor::block_on(init_future);
    }

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            hoo_engine.borrow().next_frame();
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    });
}


fn main() {
    run_gui();
}


fn run_gui() {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    selected_language: Option<Language>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    LanguageSelected(Language),
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Pick list - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::LanguageSelected(language) => {
                self.selected_language = Some(language);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let pick_list = pick_list(
            &Language::ALL[..],
            self.selected_language,
            Message::LanguageSelected,
        )
        .placeholder("Choose a language...");

        let content = column![
            vertical_space(600),
            "Which is your favorite language?",
            pick_list,
            vertical_space(600),
        ]
        .width(Length::Fill)
        .align_items(Alignment::Center)
        .spacing(10);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Rust,
    Elm,
    Ruby,
    Haskell,
    C,
    Javascript,
    Other,
}

impl Language {
    const ALL: [Language; 7] = [
        Language::C,
        Language::Elm,
        Language::Ruby,
        Language::Haskell,
        Language::Rust,
        Language::Javascript,
        Language::Other,
    ];
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Language::Rust => "Rust",
                Language::Elm => "Elm",
                Language::Ruby => "Ruby",
                Language::Haskell => "Haskell",
                Language::C => "C",
                Language::Javascript => "Javascript",
                Language::Other => "Some other language",
            }
        )
    }
}