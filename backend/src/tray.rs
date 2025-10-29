use anyhow::{Ok, Result};
use image::{DynamicImage, ImageBuffer, Rgba};
use tracing::info;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::TrayIcon;
use tray_icon::{Icon, MouseButton, TrayIconBuilder, TrayIconEvent};
use webbrowser::BrowserOptions;
use winit::{
    application::ApplicationHandler,
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
};

#[derive(Debug)]
enum UserEvent {
    TrayIconEvent(tray_icon::TrayIconEvent),
    MenuEvent(tray_icon::menu::MenuEvent),
}

struct Application {
    tray_icon: Option<TrayIcon>,
    url: String,
}

impl Application {
    pub fn new(url: String) -> Self {
        Self {
            tray_icon: None,
            url,
        }
    }

    fn new_tray_icon() -> Result<TrayIcon> {
        let menu = Menu::new();
        let show_item = MenuItem::with_id("打开主界面", "打开主界面", true, None);
        let toggle_item = MenuItem::with_id("暂停交易", "暂停交易", true, None);
        let quit_item = MenuItem::with_id("退出", "退出", true, None);

        menu.append(&show_item)?;
        menu.append(&toggle_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit_item)?;

        let icon = create_tray_icon()?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("NOF0 Trading System")
            .with_icon(icon)
            .with_menu_on_left_click(false)
            .build()?;
        Ok(tray_icon)
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if winit::event::StartCause::Init == cause {
            #[cfg(not(target_os = "linux"))]
            {
                self.tray_icon = Self::new_tray_icon().ok();
            }

            // We have to request a redraw here to have the icon actually show up.
            // Winit only exposes a redraw method on the Window so we use core-foundation directly.
            #[cfg(target_os = "macos")]
            unsafe {
                use objc2_core_foundation::{CFRunLoopGetMain, CFRunLoopWakeUp};

                let rl = CFRunLoopGetMain().unwrap();
                CFRunLoopWakeUp(&rl);
            }

            webbrowser::open_browser_with_options(
                webbrowser::Browser::Default,
                &self.url,
                BrowserOptions::new()
                    .with_target_hint("nof0")
                    .with_suppress_output(true),
            )
            .ok();
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::TrayIconEvent(tray_event) => match tray_event {
                TrayIconEvent::DoubleClick { .. } => {
                    webbrowser::open_browser_with_options(
                        webbrowser::Browser::Default,
                        &self.url,
                        BrowserOptions::new()
                            .with_target_hint("nof0")
                            .with_suppress_output(true),
                    )
                    .ok();
                }
                _ => {}
            },
            UserEvent::MenuEvent(menu_event) => match menu_event.id.as_ref() {
                id if id == "打开主界面" => {
                    webbrowser::open_browser_with_options(
                        webbrowser::Browser::Default,
                        &self.url,
                        BrowserOptions::new()
                            .with_target_hint("nof0")
                            .with_suppress_output(true),
                    )
                    .ok();
                }
                id if id == "暂停交易" => {
                    info!("Tray: Toggle trading");
                    // 处理暂停/恢复交易逻辑
                }
                id if id == "退出" => {
                    event_loop.exit();
                }
                _ => {}
            },
        }
    }
}

/// 在主线程运行系统托盘
pub fn run_system_tray(url: String) -> Result<()> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;

    // set a tray event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        proxy.send_event(UserEvent::TrayIconEvent(event)).ok();
    }));
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        proxy.send_event(UserEvent::MenuEvent(event)).ok();
    }));

    let mut app = Application::new(url);
    #[cfg(target_os = "linux")]
    std::thread::spawn(|| {
        gtk::init().unwrap();

        let _tray_icon = Application::new_tray_icon();

        gtk::main();
    });

    event_loop.run_app(&mut app)?;
    Ok(())
}

fn create_tray_icon() -> Result<Icon> {
    // 创建一个简单的 32x32 图标（蓝色圆圈）
    let size = 32u32;
    let mut img = ImageBuffer::from_fn(size, size, |x, y| {
        let dx = (x as f32 - size as f32 / 2.0).abs();
        let dy = (y as f32 - size as f32 / 2.0).abs();
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < size as f32 / 2.0 - 2.0 {
            // 蓝色
            Rgba([0, 120, 215, 255])
        } else if distance < size as f32 / 2.0 {
            // 白色边框
            Rgba([255, 255, 255, 255])
        } else {
            // 透明
            Rgba([0, 0, 0, 0])
        }
    });

    // 添加白色 "N" 字母
    draw_letter_n(&mut img, size);

    let rgba = DynamicImage::ImageRgba8(img).to_rgba8();
    let (width, height) = rgba.dimensions();
    let icon = Icon::from_rgba(rgba.into_raw(), width, height)?;

    Ok(icon)
}

fn draw_letter_n(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, size: u32) {
    let white = Rgba([255, 255, 255, 255]);

    // 简单的 "N" 字母
    let start_x = size / 4;
    let end_x = size * 3 / 4;
    let start_y = size / 4;
    let end_y = size * 3 / 4;
    let thickness = 2;

    // 左竖线
    for y in start_y..end_y {
        for dx in 0..thickness {
            if start_x + dx < size {
                img.put_pixel(start_x + dx, y, white);
            }
        }
    }

    // 斜线
    for i in 0..(end_y - start_y) {
        let x = start_x + (i * (end_x - start_x)) / (end_y - start_y);
        let y = start_y + i;
        for dx in 0..thickness {
            if x + dx < size && y < size {
                img.put_pixel(x + dx, y, white);
            }
        }
    }

    // 右竖线
    for y in start_y..end_y {
        for dx in 0..thickness {
            if end_x - thickness + dx < size {
                img.put_pixel(end_x - thickness + dx, y, white);
            }
        }
    }
}
