use iced::{Application, Column, Command, executor, Settings, Text, text_input, TextInput};
use std::io::Read;
use std::path::Path;
use std::fs::OpenOptions;

fn main() {
    MarxtMain::run(Settings::default());
}


struct MarxtMain {
    state_input_pathname: text_input::State,
    pathname: String,
    text: String,
}

#[derive(Debug, Clone)]
enum Message {
    ChangePathname(String)
}

impl Application for MarxtMain {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    
    fn new(_flags: ()) ->  (MarxtMain, Command<Message>) {
        (
            MarxtMain {
                state_input_pathname: text_input::State::default(),
                pathname: "".to_string(),
                text: "".to_string(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Marxt".to_string()
    }

    fn view(&mut self) -> iced::Element<'_, Message> {
        let text_input = TextInput::new(
            &mut self.state_input_pathname,
            "Input pathname...",
            &(self.pathname),
            Message::ChangePathname
        ).padding(5);
        let mut col = Column::new().padding(10).push(text_input);
        col = col.push(Text::new(self.text.clone()));
        col.into()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ChangePathname(pathname) => {
                self.pathname = pathname.clone();
                let cloned_pathname = pathname.clone();
                let open_result = OpenOptions::new().read(true).open(Path::new(cloned_pathname.as_str()));
                match open_result {
                    Ok(mut file) => {
                        self.text = "".to_string();
                        match file.read_to_string(&mut self.text) {
                            Ok(_) => {}
                            Err(err) => {
                                self.text = format!("Error: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        self.text = format!("Error: {}", err);
                    }
                }
            }
        }
        Command::none() 
    }
}
