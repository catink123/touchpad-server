use json::JsonValue;
use log::{warn, error};
use mouse_rs::{Mouse, types::keys::Keys};

pub fn get_client_page() -> &'static str {
  include_str!("client.html")
}

pub const ICON_DATA: &[u8; 426] = include_bytes!("touchpad-server.png");

pub fn parse_message(message: String, mouse: &Mouse) {
    match json::parse(message.as_str()) {
        Ok(data) => {
            let message_type = &data["type"];
            let message_data = &data["data"];
            if message_type.is_null() || !message_type.is_string() {
                warn!("Malformed data received, ignoring.");
                return;
            }
            match message_type {
                JsonValue::String(_) | JsonValue::Short(_) => {
                    match message_type.to_string().as_str() {
                        "move" => {
                            if message_data.is_null() {
                                warn!("Malformed data received, ignoring.");
                                return;
                            }
                            let x = &message_data["x"];
                            let y = &message_data["y"];
                            if x.is_null() || !x.is_number() || y.is_null() || !y.is_number() {
                                warn!("Parameters of function 'move' are incorrect, ignoring.");
                            }
                            if let Ok(m_pos) = mouse.get_position() {
                                mouse
                                    .move_to(
                                        m_pos.x + x.as_i32().unwrap(),
                                        m_pos.y + y.as_i32().unwrap(),
                                    )
                                    .unwrap_or_else(|_| error!("Cannot move the mouse!"));
                            } else {
                                error!("Cannot get the mouse position!");
                            }
                        }
                        "left_down" => {
                            mouse
                                .press(&Keys::LEFT)
                                .unwrap_or_else(|_| error!("Cannot press LMB!"));
                        }
                        "right_down" => {
                            mouse
                                .press(&Keys::RIGHT)
                                .unwrap_or_else(|_| error!("Cannot press RMB!"));
                        }
                        "left_up" => {
                            mouse
                                .release(&Keys::LEFT)
                                .unwrap_or_else(|_| error!("Cannot release LMB!"));
                        }
                        "right_up" => {
                            mouse
                                .release(&Keys::RIGHT)
                                .unwrap_or_else(|_| error!("Cannot release RMB!"));
                        }
                        other => warn!("Unknown message type of '{}', ignoring.", other),
                    }
                }
                other => {
                    warn!("Unknown message type of {:?}, ignoring.", other);
                }
            }
        }
        Err(_) => warn!("Couldn't parse the message, ignoring."),
    }
}