use iced::{Application, Column, Command, executor, Settings, Text, text_input, TextInput};
use std::io::Read;
use std::path::Path;
use std::fs::OpenOptions;
use std::fs;
use std::io::{BufWriter, Write};

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

#[derive(Debug, Clone)]
enum MartxFile {
    Dir,
    File,
    Unprocessable,
}

impl MarxtMain {

    /// ログファイルの場所
    fn log_path(&self) -> &str {
        "/tmp/marxt.log"
    }

    /// ログファイルに書き込む
    fn write_to_log(&self, log_path: &str, message: String) {
        let file = OpenOptions::new().create(true).append(true).open(log_path).unwrap();
        let mut f = BufWriter::new(file);
        f.write(message.as_bytes());
    }

    /// 指定したパスのファイルタイプを返す
    /// ファイルが存在しない場合 MartxFile::Unprocessable を返す
    fn file_type(&self, file_path: &str) -> MartxFile {
        let result_metadata = std::fs::metadata(file_path);
        match result_metadata {
            Err(err) => {
                //self.write_to_log(self.log_path(), err.to_string());
                MartxFile::Unprocessable
            },
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
                let file_type = self.file_type(&cloned_pathname);
                match file_type {
                    MartxFile::Dir => {
                        self.text = "".to_string();
                        let read_dir = fs::read_dir(cloned_pathname);
                        match read_dir {
                            Ok(read_dir) => {
                                for entry in read_dir.into_iter() {
                                    match entry {
                                        Ok(entry) => {
                                            self.text += &*format!("{:?}\n", entry.path());
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
                    MartxFile::File => {
                        self.text = "".to_string();

                        let open_result = OpenOptions::new().read(true).open(Path::new(cloned_pathname.as_str()));
                        match open_result {
                            Ok(mut file) => {
                                match file.read_to_string(&mut self.text) {
                                    Ok(res) => {
                                        println!("DEBUG {}", res);
                                    }
                                    Err(err) => {
                                        self.text = format!("Error: {}", err);
                                    }
                                }

                            }
                            Err(err) => {
                                self.write_to_log(self.log_path(), "Error : read_file".to_string());
                                self.write_to_log(self.log_path(), err.to_string());
                            }
                        }
                    }
                    MartxFile::Unprocessable => {
                    }
                }
            }
        }
        Command::none() 
    }
}
