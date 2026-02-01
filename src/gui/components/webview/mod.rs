use tracing::info;
use wry::WebViewBuilder;

pub fn open_webview(url: String) -> iced::Task<()> {
    let window_setting = iced::window::Settings {
        ..Default::default()
    };
    let (id, open_task) = iced::window::open(window_setting);

    let run_task = iced::window::run::<()>(id, move |window| {
        info!("Opening: {} (Window: {})", url, id);

        let builder = WebViewBuilder::new().with_url(url);
        let _webview = builder.build(&window.window_handle().unwrap()).unwrap();
    });

    iced::Task::batch(vec![open_task.map(|_| ()), run_task])
}
