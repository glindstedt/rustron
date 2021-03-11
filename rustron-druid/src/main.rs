use druid::{WindowDesc, AppLauncher, LocalizedString, Widget, Data};
use druid::widget::{Label, Button, Column, Align, Padding};

use rustron::app::App;
use rustron_lib::protocol::{ToggleOption, GlobalSetting};
use rustron_lib::protocol::DeviceId::Multicast;
use rustron_lib::protocol::NeutronMessage::SetGlobalSetting;

#[derive(Data)]
struct Context {
    app: App,
}

fn main() {
    let app = App::new();

    let main_window = WindowDesc::new(ui_builder);
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(app)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<Context> {
    // The label text will be computed based dynamically based on the current locale and count
    let text =
        LocalizedString::new("hello-counter").with_arg("count", |data: &Context, _env| (*data.app.command_history.last().unwrap_or(&String::from("?"))).into());
    let label = Label::new(text);
    let button = Button::new("increment", |_ctx, data: &mut Context, _env| {
        *data.app.command(SetGlobalSetting(Multicast, GlobalSetting::ParaphonicMode(ToggleOption::On))
            .as_bytes()
            .as_slice())
    });

    let mut col = Column::new();
    col.add_child(Align::centered(Padding::new(5.0, label)), 1.0);
    col.add_child(Padding::new(5.0, button), 1.0);
    col
}
