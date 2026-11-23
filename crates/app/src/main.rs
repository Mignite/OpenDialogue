use rod_app::editor_app::{self, EditorApp};

fn main() -> iced::Result {
    iced::application(EditorApp::nuevo, editor_app::update, editor_app::view)
        .title("Rusty Open Dialogue")
        .window_size([1600.0, 900.0])
        .subscription(editor_app::subscription)
        .run()
}
