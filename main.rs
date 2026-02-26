use qmetaobject::*;
use cstr::cstr;
use std::process::{Command};
use serde::Deserialize;
use reqwest::blocking::Client;
use base64::{Engine as _, engine::general_purpose};
use std::fs::File;
use std::io::{Write, Read};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
struct GmailList { messages: Option<Vec<GmailThread>> }
#[derive(Deserialize, Debug)]
struct GmailThread { id: String }
#[derive(Deserialize, Debug)]
struct GmailMessage { payload: GmailPayload }
#[derive(Deserialize, Debug)]
struct GmailPayload { headers: Vec<GmailHeader> }
#[derive(Deserialize, Debug)]
struct GmailHeader { name: String, value: String }

#[derive(Clone, Default)]
struct Contact { name: String, email: String, phone: String }

#[derive(QObject, Default)]
pub struct NostliBackend {
    base: qt_base_class!(trait QObject),
    pub logged_in: qt_property!(bool; READ get_logged_in WRITE set_logged_in NOTIFY logged_in_changed),
    pub user_name: qt_property!(QString; READ get_user_name WRITE set_user_name NOTIFY user_name_changed),
    pub user_email: qt_property!(QString; READ get_user_email NOTIFY user_email_changed), // READ ONLY
    pub user_phone: qt_property!(QString; READ get_user_phone WRITE set_user_phone NOTIFY user_phone_changed),
    pub user_desc: qt_property!(QString; READ get_user_desc WRITE set_user_desc NOTIFY user_desc_changed),
    pub user_avatar: qt_property!(QString; READ get_user_avatar WRITE set_user_avatar NOTIFY user_avatar_changed),
    pub messages: qt_property!(QStringList; READ get_messages NOTIFY messages_changed),
    pub active_room: qt_property!(QString; READ get_active_room WRITE set_active_room NOTIFY active_room_changed),
    pub dark_mode: qt_property!(bool; READ get_dark_mode WRITE set_dark_mode NOTIFY dark_mode_changed),
    pub contact_list: qt_property!(QStringList; READ get_contact_list NOTIFY contact_list_changed),

    pub logged_in_changed: qt_signal!(),
    pub user_name_changed: qt_signal!(),
    pub user_email_changed: qt_signal!(),
    pub user_phone_changed: qt_signal!(),
    pub user_desc_changed: qt_signal!(),
    pub user_avatar_changed: qt_signal!(),
    pub messages_changed: qt_signal!(),
    pub active_room_changed: qt_signal!(),
    pub dark_mode_changed: qt_signal!(),
    pub contact_list_changed: qt_signal!(),

    pub start_google_login: qt_method!(fn(&mut self)),
    pub handle_callback: qt_method!(fn(&mut self, url_str: String)),
    pub send_message: qt_method!(fn(&mut self, msg: String)),
    pub send_image: qt_method!(fn(&mut self, file_url: String)),
    pub call_contact: qt_method!(fn(&self, phone: String)),
    pub save_contact: qt_method!(fn(&mut self, name: String, email: String, phone: String)),
    pub get_current_phone: qt_method!(fn(&self) -> QString),
    pub logout: qt_method!(fn(&mut self)),
    pub load_from_disk: qt_method!(fn(&mut self)),

    is_authed: bool,
    is_dark: bool,
    name: String,
    email: String,
    phone: String,
    description: String,
    avatar_path: String,
    current_room: String,
    chat_history: Vec<String>,
    access_token: String,
    contacts_db: Vec<Contact>,
}

impl NostliBackend {
    fn get_storage_path() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let dir = format!("{}/.local/share/nostlichat", home);
        let _ = std::fs::create_dir_all(&dir);
        format!("{}/data.json", dir)
    }

    fn save_to_disk(&self) {
        let path = Self::get_storage_path();
        if let Ok(mut file) = File::create(path) {
            let data = serde_json::json!({
                "contacts": self.contacts_db.iter().map(|c| (c.name.clone(), c.email.clone(), c.phone.clone())).collect::<Vec<_>>(),
                "history": self.chat_history,
                "dark_mode": self.is_dark,
                "my_name": self.name,
                "my_email": self.email,
                "my_phone": self.phone,
                "my_desc": self.description,
                "my_avatar": self.avatar_path,
                "is_authed": self.is_authed,
                "access_token": self.access_token
            });
            let _ = file.write_all(data.to_string().as_bytes());
        }
    }

    pub fn load_from_disk(&mut self) {
        let path = Self::get_storage_path();
        if let Ok(mut file) = File::open(path) {
            let mut content = String::new();
            let _ = file.read_to_string(&mut content);
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                self.name = data["my_name"].as_str().unwrap_or("").to_string();
                self.email = data["my_email"].as_str().unwrap_or("").to_string();
                self.phone = data["my_phone"].as_str().unwrap_or("").to_string();
                self.description = data["my_desc"].as_str().unwrap_or("").to_string();
                self.avatar_path = data["my_avatar"].as_str().unwrap_or("").to_string();
                self.is_authed = data["is_authed"].as_bool().unwrap_or(false);
                self.is_dark = data["dark_mode"].as_bool().unwrap_or(false);
                self.access_token = data["access_token"].as_str().unwrap_or("").to_string();

                if let Some(contacts) = data["contacts"].as_array() {
                    self.contacts_db = contacts.iter().map(|c| Contact {
                        name: c[0].as_str().unwrap_or("").to_string(),
                        email: c[1].as_str().unwrap_or("").to_string(),
                        phone: c[2].as_str().unwrap_or("").to_string(),
                    }).collect();
                }
                if let Some(history) = data["history"].as_array() {
                    self.chat_history = history.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect();
                }
            }
        }
        self.logged_in_changed();
        self.user_name_changed();
        self.user_email_changed();
        self.user_phone_changed();
        self.user_desc_changed();
        self.user_avatar_changed();
        self.contact_list_changed();
        self.messages_changed();
        self.dark_mode_changed();
    }

    fn get_logged_in(&self) -> bool { self.is_authed }
    fn set_logged_in(&mut self, v: bool) { self.is_authed = v; self.logged_in_changed(); self.save_to_disk(); }
    fn get_user_name(&self) -> QString { self.name.clone().into() }
    fn set_user_name(&mut self, v: QString) { self.name = v.to_string(); self.user_name_changed(); self.save_to_disk(); }
    fn get_user_email(&self) -> QString { self.email.clone().into() }
    fn get_user_phone(&self) -> QString { self.phone.clone().into() }
    fn set_user_phone(&mut self, v: QString) { self.phone = v.to_string(); self.user_phone_changed(); self.save_to_disk(); }
    fn get_user_desc(&self) -> QString { self.description.clone().into() }
    fn set_user_desc(&mut self, v: QString) { self.description = v.to_string(); self.user_desc_changed(); self.save_to_disk(); }
    fn get_user_avatar(&self) -> QString { self.avatar_path.clone().into() }
    fn set_user_avatar(&mut self, v: QString) { self.avatar_path = v.to_string(); self.user_avatar_changed(); self.save_to_disk(); }
    fn get_dark_mode(&self) -> bool { self.is_dark }
    fn set_dark_mode(&mut self, v: bool) { self.is_dark = v; self.dark_mode_changed(); self.save_to_disk(); }
    fn get_active_room(&self) -> QString { self.current_room.clone().into() }
    fn get_messages(&self) -> QStringList {
        let mut list = QStringList::default();
        for m in &self.chat_history { list.push(m.as_str().into()); }
        list
    }
    fn get_contact_list(&self) -> QStringList {
        let mut list = QStringList::default();
        for c in &self.contacts_db { list.push(c.name.as_str().into()); }
        list
    }

    fn logout(&mut self) {
        self.access_token = String::new();
        self.is_authed = false;
        let _ = std::fs::remove_file(Self::get_storage_path());
        self.logged_in_changed();
    }

    fn set_active_room(&mut self, room: QString) {
        self.current_room = room.to_string();
        self.active_room_changed();
        self.fetch_gmail();
    }

    fn send_message(&mut self, msg: String) {
        if msg.trim().is_empty() || self.access_token.is_empty() { return; }
        let recipient = self.contacts_db.iter().find(|c| c.name == self.current_room).map(|c| c.email.clone()).unwrap_or_default();
        self.chat_history.push(format!("Me: {}", msg));
        self.messages_changed();
        self.save_to_disk();

        if recipient.is_empty() { return; }
        let raw = format!("To: {}\r\nSubject: Nostlichat\r\n\r\n{}", recipient, msg);
        let encoded = general_purpose::URL_SAFE_NO_PAD.encode(raw);
        let _ = Client::new().post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
            .bearer_auth(&self.access_token).json(&serde_json::json!({ "raw": encoded })).send();
    }

    fn fetch_gmail(&mut self) {
        if self.access_token.is_empty() { return; }
        let client = Client::new();
        if let Ok(res) = client.get("https://gmail.googleapis.com/gmail/v1/users/me/messages?maxResults=10&q=is:anywhere").bearer_auth(&self.access_token).send() {
            if let Ok(json) = res.json::<GmailList>() {
                if let Some(msgs) = json.messages {
                    for m in msgs {
                        if let Ok(d_res) = client.get(format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}", m.id)).bearer_auth(&self.access_token).send() {
                            if let Ok(det) = d_res.json::<GmailMessage>() {
                                let sub = det.payload.headers.iter().find(|h| h.name == "Subject").map(|h| h.value.clone()).unwrap_or_else(|| "No Subject".into());
                                let from = det.payload.headers.iter().find(|h| h.name == "From").map(|h| h.value.clone()).unwrap_or_default();
                                let display = if from.contains(&self.email) { "Me".to_string() } else { from };
                                let formatted = format!("{}: {}", display, sub);
                                if !self.chat_history.contains(&formatted) {
                                    self.chat_history.push(formatted);
                                    self.messages_changed();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn save_contact(&mut self, name: String, email: String, phone: String) {
        self.contacts_db.push(Contact { name, email, phone });
        self.contact_list_changed();
        self.save_to_disk();
    }

    fn send_image(&mut self, file_url: String) {
        let path = file_url.replace("file://", "");
        if let Ok(bytes) = std::fs::read(&path) {
            let base64_img = general_purpose::STANDARD.encode(bytes);
            let recipient = self.contacts_db.iter().find(|c| c.name == self.current_room).map(|c| c.email.clone()).unwrap_or_default();
            if recipient.is_empty() || self.access_token.is_empty() { return; }
            let boundary = "boundary_nostli";
            let raw = format!("To: {}\r\nSubject: Photo from Nostlichat\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary={}\r\n\r\n--{}\r\nContent-Type: text/plain\r\n\r\nSent an image via Nostlichat.\r\n--{}\r\nContent-Type: image/png\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n--{}--", recipient, boundary, boundary, boundary, base64_img, boundary);
            let encoded = general_purpose::URL_SAFE_NO_PAD.encode(raw);
            let _ = Client::new().post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send").bearer_auth(&self.access_token).json(&serde_json::json!({ "raw": encoded })).send();
            self.chat_history.push("Me: 🖼️ (Image Sent)".into());
            self.messages_changed();
        }
    }

    fn get_current_phone(&self) -> QString {
        self.contacts_db.iter().find(|c| c.name == self.current_room).map(|c| c.phone.clone()).unwrap_or(self.phone.clone()).into()
    }
    
    fn call_contact(&self, phone: String) {
        let _ = Command::new("xdg-open").arg(format!("tel:{}", phone)).spawn();
    }
    
    fn start_google_login(&mut self) {
        let url = "https://turkwopxapivcllruvzo.supabase.co/auth/v1/authorize?provider=google&redirect_to=nostlichat://login-callback&scopes=https://www.googleapis.com/auth/gmail.readonly%20https://www.googleapis.com/auth/gmail.send";
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
    
    fn handle_callback(&mut self, url_str: String) {
        let key = if url_str.contains("provider_token=") { "provider_token=" } else { "access_token=" };
        if let Some(pos) = url_str.find(key) {
            let start = pos + key.len();
            let end = url_str[start..].find('&').unwrap_or(url_str.len() - start);
            self.access_token = url_str[start..start+end].to_string();
            self.set_logged_in(true);
            self.set_active_room("General".into());
            self.save_to_disk();
        }
    }
}

fn main() {
    qml_register_type::<NostliBackend>(cstr!("NostliBackend"), 1, 0, cstr!("NostliBackend"));
    let mut engine = QmlEngine::new();
    let mut backend = NostliBackend::default();
    backend.load_from_disk();
    let backend_ptr = QObjectBox::new(backend);
    engine.set_object_property("backend".into(), backend_ptr.pinned());

    // 1. Get the directory where the binary is
    let mut exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    exe_path.pop(); 

    // 2. Get the Current Working Directory (CWD)
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // 3. List of search paths (Absolute + Relative)
    let search_targets = [
        cwd.join("Main.qml"),
        cwd.join("qml/Main.qml"),
        exe_path.join("Main.qml"),
        exe_path.join("../Main.qml"),
        PathBuf::from("Main.qml"),
        PathBuf::from("/home/phablet/Main.qml"), // UT standard
    ];

    let mut found_path = String::new();

    println!("--- Searching for Main.qml ---");
    for path in &search_targets {
        println!("Checking: {:?}", path);
        if path.exists() {
            found_path = path.to_string_lossy().to_string();
            println!(">> FOUND AT: {}", found_path);
            break;
        }
    }

    if found_path.is_empty() {
        eprintln!("CRITICAL ERROR: Main.qml not found! Is it in your project root?");
        return;
    }

    engine.load_file(found_path.into());
    engine.exec();
}
