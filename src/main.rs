// Marxt : Markdown viewer

use iced::{Application, Column, Command, executor, Padding, Settings, Text, text_input, TextInput};
use std::path::Path;
use std::fs::OpenOptions;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use maplit::hashmap;

const FONT_NORMAL: u16 = 20;
const FONT_H5: u16 = 22;
const FONT_H4: u16 = 24;
const FONT_H3: u16 = 26;
const FONT_H2: u16 = 28;
const FONT_H1: u16 = 30;

const PADDING_NORMAL: Padding = Padding::new(5);

pub fn main() -> iced::Result {
    MarxtMain::run(
        Settings::default()
    )
}

struct MarxtMain {

    /// Text in the text input widget.
    state_input_pathname: text_input::State,

    marxt_file: MarxtResource,

    /// Path for read directory or file.
    pathname: String,

    /// Contents in the read file or entries in the read directory.
    list_text: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {

    /// Message for change `pathname`.
    /// This message send after change `state_input_pathname`.
    ChangePathname(String)
}

/// Marxt original file category
#[derive(Debug, Clone)]
enum MarxtResource {

    /// Directory
    Dir(Vec<String>),

    /// File
    File(Vec<String>),

    /// Not exists file or directory, Unreadable file or directory, etc
    Unprocessable,
}

impl MarxtResource {
    fn from(path: &str) -> Option<Self> {
        return match fs::metadata(path) {
            Err(_) => {
                None
            }
            Ok(metadata) => {
                if metadata.is_file() {
                    let mut got_lines = vec![];
                    let open_result = OpenOptions::new().read(true).open(Path::new(path.clone()));
                    match open_result {
                        Ok(file) => {
                            let reader = BufReader::new(file);
                            let lines = reader.lines();

                            for line in lines.into_iter() {
                                got_lines.push(line.unwrap());
                            };

                            Some(Self::File(got_lines))
                        }
                        Err(_err) => {
                            Some(Self::Unprocessable)
                        }
                    }
                } else if metadata.is_dir() {
                    let read_dir = fs::read_dir(path.clone());
                    let mut entries = vec![];
                    match read_dir {
                        Ok(read_dir) => {
                            for entry in read_dir.into_iter() {
                                match entry {
                                    Ok(entry) => {
                                        entries.push(entry.path().to_str().unwrap().to_string());
                                    }
                                    Err(_err) => {}
                                }
                            }
                            Some(Self::Dir(entries))
                        }
                        Err(_err) => {
                            None
                        }
                    }
                } else {
                    None
                }
            }
        }
    }

    fn parse(&self, line: String) -> Parsed {
        let rules = hashmap! {
                        "#".to_owned() => FONT_H1,
                        "##".to_owned() => FONT_H2,
                        "###".to_owned() => FONT_H3,
                        "####".to_owned() => FONT_H4,
                        "#####".to_owned() => FONT_H5,
                    };
        match self {
            MarxtResource::Dir(_) => {
                Parsed::new(line, FONT_NORMAL)
            }
            MarxtResource::File(_) => {
                let first_word = line.split_whitespace().nth(0);
                return match first_word {
                    None => {
                        return Parsed::new(line, FONT_NORMAL);
                    }
                    Some(first_word) => {
                        let matched_size = rules.get(first_word);
                        match matched_size {
                            None => {
                                Parsed::new(line, FONT_NORMAL)
                            }
                            Some(got_size) => {
                                Parsed::new(
                                    line.replace(first_word, ""),
                                    *got_size,
                                )
                            }
                        }
                    }
                };
            }
            MarxtResource::Unprocessable => {
                Parsed::new(line, FONT_NORMAL)
            }
        }
    }
}

/// Parsed line and font size
struct Parsed {

    /// Parsed line after remove keyword for markup.
    line: String,

    /// Font size
    size: u16,
}

impl Parsed {

    /// * `line` - Parsed line after remove keyword for markup.
    /// * `size` - Font size
    fn new(line: String, size: u16) -> Parsed {
        Parsed {
            line,
            size,
        }
    }
}

impl MarxtMain {

    /// Log file path of this application.
    fn log_path(&self) -> &str {
        "/tmp/marxt.log"
    }

    /// Write to log file.
    ///
    /// * `log_path` - Log file path
    /// * `message` - Message to write
    fn write_to_log(&self, log_path: &str, message: String) {
        let file = OpenOptions::new().create(true).append(true).open(log_path).unwrap();
        let mut f = BufWriter::new(file);
        f.write(message.as_bytes()).unwrap();
    }
}

impl Application for MarxtMain {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (MarxtMain, Command<Message>) {
        (
            MarxtMain {
                state_input_pathname: text_input::State::default(),
                pathname: "".to_string(),
                list_text: vec![],
                marxt_file: MarxtResource::Unprocessable,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Marxt".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ChangePathname(pathname) => {
                self.pathname = pathname.clone();
                let cloned_pathname = pathname.clone();
                match MarxtResource::from(&cloned_pathname) {
                    None => {}
                    Some(resource) => {
                        match resource {
                            MarxtResource::Dir(entries) => {
                                self.list_text = entries;
                            }
                            MarxtResource::File(lines) => {
                                self.list_text = lines;
                            }
                            MarxtResource::Unprocessable => {}
                        }
                    }
                }
            }
        }
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Message> {
        self.write_to_log(self.log_path(), "view".to_string());
        let text_input = TextInput::new(
            &mut self.state_input_pathname,
            "Input pathname...",
            &(self.pathname),
            Message::ChangePathname,
        ).padding(PADDING_NORMAL);
        let mut col = Column::new().padding(PADDING_NORMAL).push(text_input);
        for text in self.list_text.iter() {
            // let parsed = self.markup_rules.parse(text.to_string());
            let parsed = self.marxt_file.parse(text.to_string());
            col = col.push(Text::new(parsed.line).size(parsed.size));
        }
        col.into()
    }
}
