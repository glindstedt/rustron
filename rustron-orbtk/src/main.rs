use log::info;
use orbtk::prelude::*;

#[derive(Copy, Clone, Debug)]
enum Action {
    Connect,
}

#[derive(Default, AsAny)]
struct MainViewState {
    action: Option<Action>,
    connected: bool,
}

impl MainViewState {
    fn action(&mut self, action: impl Into<Option<Action>>) {
        self.action = action.into();
    }

    fn connect(&mut self) {
        info!("connect");
        self.connected = !self.connected
    }
}

impl State for MainViewState {
    fn init(&mut self, _registry: &mut Registry, _ctx: &mut Context) {
        info!("init")
    }

    fn update(&mut self, _: &mut Registry, ctx: &mut Context) {
        info!("update");
        if let Some(action) = self.action {
            match action {
                Action::Connect => {
                    let text: String16 = if self.connected {
                        "Connected!".into()
                    } else {
                        "Not connected.".into()
                    };
                    // TextBlock::text_mut(&mut ctx.child("connect_state")).push("text");
                    ctx.child("connect_state").set("text", text);
                    // ctx.widget().set("text", text);
                }
            }
            self.action = None
        }
    }
}

fn main() {

    env_logger::init();

    Application::new()
        .window(|ctx| {
            Window::new()
                .title("OrbTk - minimal example")
                .position((100.0, 100.0))
                .size(420.0, 730.0)
                .child(MainView::new().build(ctx))
                .build(ctx)
        })
        .run();
}

widget!(MainView<MainViewState> {
    text: String16
});

impl Template for MainView {

    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        self.child(
            TabWidget::new()
                .close_button(false)
                .tab("Main", Container::new()
                    .margin(5)
                    .child(Grid::new()
                        .columns(Columns::create().push(100).push(100))
                        .child(
                            Button::new()
                                .attach(Grid::column(0))
                                .text("Connect")
                                .on_click(move |states, _point| {
                                    states
                                        .get_mut::<MainViewState>(id)
                                        .action(Action::Connect);
                                    true
                                })
                                .build(ctx)
                        )
                        .child(
                            TextBlock::new()
                                .attach(Grid::column(1))
                                .id("connect_state")
                                .build(ctx)
                        ).build(ctx)
                    ).build(ctx)
                )
                .tab("Log", LogView::new().build(ctx))
                .build(ctx)
        )
    }

    fn render_object(&self) -> Box<dyn RenderObject> {
        Box::new(DefaultRenderObject)
    }

    fn layout(&self) -> Box<dyn Layout> {
        Box::new(GridLayout::new())
    }
}

widget!(LogView {});

impl Template for LogView {
    fn template(self, _id: Entity, ctx: &mut BuildContext) -> Self {
        self.child(TextBox::new().text("Loooogs").build(ctx))
    }
}