// Marxt : Markdown viewer

use std::collections::HashMap;
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

    /// Path for read directory or file.
    pathname: String,

    /// Contents in the read file or entries in the read directory.
    list_text: Vec<String>,

    /// Rules for render text.
    markup_rules: MarkupRules,
}

#[derive(Debug, Clone)]
enum Message {

    /// Message for change `pathname`.
    /// This message send after change `state_input_pathname`.
    ChangePathname(String)
}

/// Marxt original file category
#[derive(Debug, Clone)]
enum MartxFile {

    /// Directory
    Dir,

    /// File
    File,

    /// Not exists file or directory, Unreadable file or directory, etc
    Unprocessable,
}


/// Rules for markup text.
struct MarkupRules {

    /// Include prefix and font size.
    rules: HashMap<String, u16>,
}

impl MarkupRules {

    /// * `rules` - Prefix and font size
    fn new(rules: HashMap<String, u16>) -> MarkupRules {
        MarkupRules {
            rules
        }
    }

    /// Parse target line with owned rule.
    ///
    /// * `line` - Parse target line
    fn parse(&self, line: String) -> Parsed {
        let first_word = line.split_whitespace().nth(0);
        return match first_word {
            None => {
                return Parsed::new(line, FONT_NORMAL);
            }
            Some(first_word) => {
                let matched_size = self.rules.get(first_word);
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

    /// Judge file type
    ///
    /// * `file_path` - Path of target file
    fn file_type(&self, file_path: &str) -> MartxFile {
        let result_metadata = fs::metadata(file_path);
        match result_metadata {
            Err(err) => {
                self.write_to_log(self.log_path(), err.to_string());
                MartxFile::Unprocessable
            }
            Ok(metadata) => {
                if metadata.is_file() {
                    return MartxFile::File;
                }
                if metadata.is_dir() {
                    return MartxFile::Dir;
                }
                MartxFile::Unprocessable
            }
        }
    }

    fn update_read_dir(&mut self, cloned_pathname: String) {
        let read_dir = fs::read_dir(cloned_pathname);
        match read_dir {
            Ok(read_dir) => {
                self.list_text = vec![];
                for entry in read_dir.into_iter() {
                    match entry {
                        Ok(entry) => {
                            self.list_text.push(entry.path().to_str().unwrap().to_string());
                        }
                        Err(err) => {
                            self.write_to_log(self.log_path(), "Error : DirEntry".to_string());
                            self.write_to_log(self.log_path(), err.to_string());
                        }
                    }
                }
            }
            Err(err) => {
                self.write_to_log(self.log_path(), "Error : read_dir".to_string());
                self.write_to_log(self.log_path(), err.to_string());
            }
        }
    }

    fn update_for_view_file(&mut self, cloned_pathname: String) {
        let open_result = OpenOptions::new().read(true).open(Path::new(cloned_pathname.as_str()));
        match open_result {
            Ok(file) => {
                let reader = BufReader::new(file);
                let lines = reader.lines();
                self.list_text = vec![];
                for line in lines.into_iter() {
                    match line {
                        Ok(line) => {
                            self.list_text.push(line);
                        }
                        Err(err) => {
                            self.write_to_log(self.log_path(), "Error : read_line".to_string());
                            self.write_to_log(self.log_path(), err.to_string());
                        }
                    }
                };
            }
            Err(err) => {
                self.write_to_log(self.log_path(), "Error : read_file".to_string());
                self.write_to_log(self.log_path(), err.to_string());
            }
        }
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
                markup_rules: MarkupRules::new(
                    hashmap! {
                        "#".to_owned() => FONT_H1,
                        "##".to_owned() => FONT_H2,
                        "###".to_owned() => FONT_H3,
                        "####".to_owned() => FONT_H4,
                        "#####".to_owned() => FONT_H5,
                    }
                ),
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
                let file_type = self.file_type(&cloned_pathname);
                match file_type {
                    MartxFile::Dir => {
                        self.update_read_dir(cloned_pathname);
                    }
                    MartxFile::File => {
                        self.update_for_view_file(cloned_pathname);
                    }
                    MartxFile::Unprocessable => {}
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
            let parsed = self.markup_rules.parse(text.to_string());
            col = col.push(Text::new(parsed.line).size(parsed.size));
        }
        col.into()
    }
}
