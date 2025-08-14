use std::f64::consts::{PI, TAU};

use egui::{CollapsingHeader, Color32, Pos2, Rect, Shape};
use num_complex::Complex32;
use rand::Rng as _;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    fractal_pendulum_app: FractalPendulumApp,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 获取每帧耗时
        self.fractal_pendulum_app.data.frame_time =
            (frame.info().cpu_usage.unwrap_or_default() * 1000.0) as u32;

        // 底部菜单
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.menu_button("关于", |ui| {
                        ui.add(egui::github_link_file!(
                            "https://github.com/LemoMew/FractalPendulum/",
                            "项目地址"
                        ));
                        powered_by_egui_and_eframe(ui);
                    });
                    ui.menu_button("提示", |ui| {
                        ui.label("一些变量可以输入超出拖动条范围的数字");
                        ui.label("有时数字输入负数会出问题，懒得管了，重置就行");
                        ui.label("数值计算错误可以通过缩短迭代步长缓解");
                        ui.label("角速度很快时看起来运动会比较怪");
                        ui.label("");
                        ui.label("动态色相计算方式：在目标值1、2之间插值，");
                        ui.label("乘以倍率，最后整体偏移目标值3");
                    });
                    egui::warn_if_debug_build(ui);
                });
            });
        });

        // 主体
        egui::CentralPanel::default().show(ctx, |ui| {
            self.fractal_pendulum_app.ui(ui);
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

// -------- -------- -------- -------- -------- -------- -------- --------

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct FractalPendulumApp {
    // 要保存的设置放在这里
    setting: FractalPendulumAppSetting,

    // 其他数值在这里
    #[serde(skip)]
    data: FractalPendulumAppData,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct FractalPendulumAppSetting {
    paused: bool,
    m: [f64; 3],
    l: [f64; 3],
    q: [f64; 6],
    g: f64,
    delta_t: f64,
    h: f64,
    show_balls: bool,
    ball_radius: f32,
    depth: usize,
    zoom: f32,
    x_offset: f32,
    y_offset: f32,
    line_width: f32,
    width_decay: f32,
    hue_mode: HueMode,
    hue1: f32,
    hue2: f32,
    hue_target1: HueTarget,
    hue_target2: HueTarget,
    hue_target3: HueTarget,
    hue_factor: f32,
    saturation: f32,
    saturation_decay: f32,
    luminance: f32,
    luminance_decay: f32,
}

impl Default for FractalPendulumAppSetting {
    fn default() -> Self {
        Self {
            paused: false,
            m: [1.0, 0.5, 0.3],
            l: [1.0, 0.9, 0.8],
            q: [-3.0, 0.5, -0.3, -1.0, 0.5, 1.0],
            g: 9.8,
            delta_t: 0.001,
            h: 0.001,
            show_balls: true,
            ball_radius: 10.0,
            depth: 12,
            zoom: 0.1,
            x_offset: 0.0,
            y_offset: 0.0,
            line_width: 5.0,
            width_decay: 0.8,
            hue_mode: HueMode::Dynamic,
            hue1: 0.0,
            hue2: std::f32::consts::TAU,
            hue_target1: HueTarget::Omega1,
            hue_target2: HueTarget::Omega2,
            hue_target3: HueTarget::Theta1,
            hue_factor: 0.1,
            saturation: 1.0,
            saturation_decay: 0.99,
            luminance: 1.0,
            luminance_decay: 0.9,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
enum HueMode {
    Fixed,
    Dynamic,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
enum HueTarget {
    Omega1,
    Omega2,
    Omega3,
    Theta1,
    Theta2,
    Theta3,
}

struct FractalPendulumAppData {
    opacity: f32,
    line_count: usize,
    frame_time: u32,
    debug_message: String,
    t: f64,
    v: f64,
    e: f64,
}

impl Default for FractalPendulumApp {
    fn default() -> Self {
        Self {
            setting: FractalPendulumAppSetting::default(),

            data: FractalPendulumAppData {
                opacity: 1.0,
                line_count: 0,
                frame_time: 0,
                debug_message: "正常".to_owned(),
                t: 0.0,
                v: 0.0,
                e: 0.0,
            },
        }
    }
}

impl eframe::App for FractalPendulumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::dark_canvas(&ctx.style())
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(0),
            )
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }
}

impl FractalPendulumApp {
    // 显示内容
    fn ui(&mut self, ui: &mut egui::Ui) {
        // 没有暂停时，一直请求重绘并且迭代微分方程
        if !self.setting.paused {
            ui.ctx().request_repaint();
            let ode = Ode::new(self);
            let mut stepper = ode_solvers::Dop853::new(
                ode,
                0.0,
                self.setting.delta_t,
                self.setting.h,
                State::new(
                    self.setting.q[0],
                    self.setting.q[1],
                    self.setting.q[2],
                    self.setting.q[3],
                    self.setting.q[4],
                    self.setting.q[5],
                ),
                1e-12,
                1e-12,
            );
            let res = stepper.integrate();
            if res.is_ok() {
                let y = stepper.y_out().last().expect("数值计算的结果应当存在");
                self.setting.q = [y[0], y[1], y[2], y[3], y[4], y[5]];
                for i in [0, 2, 4] {
                    self.setting.q[i] = self.setting.q[i].rem_euclid(TAU);
                    if self.setting.q[i] > PI {
                        self.setting.q[i] -= TAU;
                    }
                }

                let g = self.setting.g;
                let [l1, l2, l3] = self.setting.l;
                let [m1, m2, m3] = self.setting.m;
                let [q1, q2, q3, q4, q5, q6] = self.setting.q;

                self.data.t = 0.5 * (m1 + m2 + m3) * l1 * l1 * q2 * q2
                    + 0.5 * m2 * l2 * l2 * q4 * q4
                    + 0.5 * m3 * l3 * l3 * q6 * q6
                    + m2 * l1 * l2 * q3.cos() * q2 * q4
                    + m3 * l1 * l3 * q5.cos() * q2 * q6;
                self.data.v = -(m1 + m2 + m3) * g * l1 * q1.cos()
                    - m2 * g * l2 * (q1 + q3).cos()
                    - m3 * g * l3 * (q1 + q5).cos();
                self.data.e = self.data.t + self.data.v;
            } else {
                self.setting.paused = true;
                self.data.debug_message = "数值计算错误，已暂停".to_owned();
            }
        }

        // 绘制图案
        let painter = egui::Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );
        self.paint(&painter);
        ui.expand_to_include_rect(painter.clip_rect());

        // 绘制设置界面，对其整体应用不透明度，可以折叠到一行
        ui.multiply_opacity(self.data.opacity);
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.set_max_width(270.0);
            CollapsingHeader::new("选项").show(ui, |ui| self.options_ui(ui));
        });
    }

    #[expect(clippy::too_many_lines)]
    // 比较长的画分形函数
    fn paint(&mut self, painter: &egui::Painter) {
        // 使用起点+向量的形式保存线段，复数便于表示分形迭代时的关系
        struct Node {
            start: Complex32,
            vec: Complex32,
        }

        impl Node {
            fn apply(&self, transform: Complex32) -> Self {
                Self {
                    start: self.start + self.vec,
                    vec: self.vec * transform,
                }
            }
        }

        // 改个名方便说话
        let l1 = self.setting.l[0] as f32;
        let l2 = self.setting.l[1] as f32;
        let l3 = self.setting.l[2] as f32;
        let t1 = self.setting.q[0] as f32;
        let t2 = self.setting.q[2] as f32;
        let t3 = self.setting.q[4] as f32;

        // 色相由起点终点插值得到，根据模式的不同选择起点终点
        let h1;
        let h2;
        match self.setting.hue_mode {
            HueMode::Fixed => {
                h1 = self.setting.hue1;
                h2 = self.setting.hue2;
            }
            HueMode::Dynamic => {
                let enum_to_value = |target: &HueTarget| match target {
                    HueTarget::Omega1 => self.setting.q[1],
                    HueTarget::Omega2 => self.setting.q[3],
                    HueTarget::Omega3 => self.setting.q[5],
                    HueTarget::Theta1 => self.setting.q[0],
                    HueTarget::Theta2 => self.setting.q[2],
                    HueTarget::Theta3 => self.setting.q[4],
                } as f32;

                let h = enum_to_value(&self.setting.hue_target3);
                h1 = h + enum_to_value(&self.setting.hue_target1) * self.setting.hue_factor;
                h2 = h + enum_to_value(&self.setting.hue_target2) * self.setting.hue_factor;
            }
        }

        // 线段迭代关系
        let transforms = [
            Complex32::from_polar(l2 / l1, t2),
            Complex32::from_polar(l3 / l1, t3),
        ];

        // 缩放到屏幕的坐标变换
        let rect = painter.clip_rect();
        let to_screen = egui::emath::RectTransform::from_to(
            Rect::from_center_size(Pos2::ZERO, rect.square_proportions() / self.setting.zoom),
            rect,
        );

        // 迭代过程中用到的变量
        let mut shapes: Vec<Shape> = Vec::new();

        let mut nodes: Vec<Node> = Vec::new();
        nodes.push(Node {
            start: Complex32::new(self.setting.x_offset, self.setting.y_offset),
            vec: Complex32::from_polar(l1, t1 + std::f32::consts::PI / 2.0),
        });
        let mut new_nodes: Vec<Node> = Vec::new();

        let mut width = self.setting.line_width;
        let mut luminance = self.setting.luminance;
        let mut saturation = self.setting.saturation;

        // 画球
        if self.setting.show_balls {
            let mut ball_nodes: Vec<Node> = Vec::new();
            ball_nodes.push(Node {
                start: Complex32::new(self.setting.x_offset, self.setting.y_offset),
                vec: Complex32::from_polar(l1, t1 + std::f32::consts::PI / 2.0),
            });
            for &transform in &transforms {
                ball_nodes.push(ball_nodes[0].apply(transform));
            }

            for (i, ball) in ball_nodes.iter().enumerate() {
                let end = ball.start + ball.vec;
                shapes.push(Shape::circle_filled(
                    to_screen * Pos2::new(end.re, end.im),
                    self.setting.m[i].sqrt() as f32 * self.setting.ball_radius,
                    hsl_to_rgb(
                        lerp(h1, h2, 0.5),
                        saturation * self.setting.saturation_decay.powi(i as i32 + 1),
                        luminance * self.setting.luminance_decay.powi(i as i32 + 1),
                    ),
                ));
            }
        }

        // 画线段工具
        let mut paint_line = |node: &Node, color: Color32, width: f32| {
            let a = node.start;
            let b = node.start + node.vec;
            let line = [
                to_screen * Pos2::new(a.re, a.im),
                to_screen * Pos2::new(b.re, b.im),
            ];

            if rect.intersects(Rect::from_two_pos(line[0], line[1])) {
                shapes.push(Shape::line_segment(line, (width, color)));
            }
        };

        // 画线段，迭代
        for _ in 1..=self.setting.depth {
            new_nodes.clear();
            new_nodes.reserve(nodes.len() * 2);

            width *= self.setting.width_decay;
            luminance *= self.setting.luminance_decay;
            saturation *= self.setting.saturation_decay;

            for (i, a) in nodes.iter().enumerate() {
                paint_line(
                    a,
                    hsl_to_rgb(
                        lerp(h1, h2, (i as f32 + 0.5) / nodes.len() as f32),
                        saturation,
                        luminance,
                    ),
                    width,
                );
                for &transform in &transforms {
                    new_nodes.push(a.apply(transform));
                }
            }

            std::mem::swap(&mut nodes, &mut new_nodes);
        }

        // 少画的补上
        for (i, a) in nodes.iter().enumerate() {
            paint_line(
                a,
                hsl_to_rgb(
                    lerp(h1, h2, (i as f32 + 0.5) / nodes.len() as f32),
                    saturation,
                    luminance,
                ),
                width,
            );
        }

        // 统计线段数
        self.data.line_count = if self.setting.show_balls {
            shapes.len() - 3
        } else {
            shapes.len()
        };

        // 先画颜色深的，否则会显脏
        painter.extend(shapes.into_iter().rev());
    }

    #[expect(clippy::too_many_lines)]
    // 又臭又长的画设置界面函数，用不着注释，对着成品看就是
    fn options_ui(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.setting.paused, "暂停");

        ui.add(
            egui::DragValue::new(&mut self.data.opacity)
                .speed(0.01)
                .range(0.0..=1.0)
                .prefix("不透明度："),
        );

        CollapsingHeader::new("变量").show(ui, |ui| {
            egui::Grid::new("变量网格")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("θ1");
                    ui.add(
                        egui::Slider::new(&mut self.setting.q[0], -PI..=PI)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("θ2");
                    ui.add(
                        egui::Slider::new(&mut self.setting.q[2], -PI..=PI)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("θ3");
                    ui.add(
                        egui::Slider::new(&mut self.setting.q[4], -PI..=PI)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("ω1");
                    ui.add(
                        egui::Slider::new(&mut self.setting.q[1], -10.0..=10.0)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("ω2");
                    ui.add(
                        egui::Slider::new(&mut self.setting.q[3], -10.0..=10.0)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("ω3");
                    ui.add(
                        egui::Slider::new(&mut self.setting.q[5], -10.0..=10.0)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();
                });

            if ui.button("随机").clicked() {
                let mut rng = rand::rng();
                for qi in &mut self.setting.q {
                    *qi = rng.random_range(-PI..=PI);
                }
            }
        });

        CollapsingHeader::new("常量").show(ui, |ui| {
            egui::Grid::new("常量网格")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("m1");
                    ui.add(
                        egui::Slider::new(&mut self.setting.m[0], 0.1..=10.0)
                            .logarithmic(true)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("m2");
                    ui.add(
                        egui::Slider::new(&mut self.setting.m[1], 0.1..=10.0)
                            .logarithmic(true)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("m3");
                    ui.add(
                        egui::Slider::new(&mut self.setting.m[2], 0.1..=10.0)
                            .logarithmic(true)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("l1");
                    ui.add(
                        egui::Slider::new(&mut self.setting.l[0], 0.5..=3.0)
                            .logarithmic(true)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("l2");
                    ui.add(
                        egui::Slider::new(&mut self.setting.l[1], 0.5..=3.0)
                            .logarithmic(true)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("l3");
                    ui.add(
                        egui::Slider::new(&mut self.setting.l[2], 0.5..=3.0)
                            .logarithmic(true)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("g");
                    ui.add(
                        egui::Slider::new(&mut self.setting.g, 0.1..=100.0)
                            .logarithmic(true)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("Δt").on_hover_text(
                        "每帧之间的时间间隔，帧率为60时，0.0166...7对应现实时间流速",
                    );
                    ui.add(
                        egui::Slider::new(&mut self.setting.delta_t, 0.0001..=0.01667)
                            .logarithmic(true)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("h").on_hover_text("迭代步长");
                    ui.add(
                        egui::Slider::new(&mut self.setting.h, 0.0001..=0.01)
                            .logarithmic(true)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();
                });

            if ui.button("随机").clicked() {
                let mut rng = rand::rng();
                for mi in &mut self.setting.m {
                    *mi = rng.random_range(0.1..10.0);
                }
                for li in &mut self.setting.l {
                    *li = rng.random_range(0.5..=3.0);
                }
            }
        });

        CollapsingHeader::new("渲染").show(ui, |ui| {
            egui::Grid::new("渲染网格")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("渲染小球");
                    ui.add(egui::widgets::Checkbox::new(
                        &mut self.setting.show_balls,
                        "",
                    ));
                    ui.end_row();

                    ui.label("半径倍率");
                    ui.add(
                        egui::Slider::new(&mut self.setting.ball_radius, 1.0..=100.0)
                            .logarithmic(true)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("递归深度");
                    ui.add(egui::Slider::new(&mut self.setting.depth, 1..=20))
                        .on_hover_text("⚠数值调高可能会非常卡");
                    ui.end_row();

                    ui.label("缩放倍率");
                    ui.add(
                        egui::Slider::new(&mut self.setting.zoom, 0.01..=1.0)
                            .logarithmic(true)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("x轴偏置");
                    ui.add(
                        egui::Slider::new(&mut self.setting.x_offset, -1.0..=1.0)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("y轴偏置");
                    ui.add(
                        egui::Slider::new(&mut self.setting.y_offset, -1.0..=1.0)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("起始宽度");
                    ui.add(
                        egui::Slider::new(&mut self.setting.line_width, 0.5..=5.0)
                            .clamping(egui::SliderClamping::Never),
                    );
                    ui.end_row();

                    ui.label("宽度衰减");
                    ui.add(egui::Slider::new(&mut self.setting.width_decay, 0.0..=1.0));
                    ui.end_row();

                    ui.label("起始亮度");
                    ui.add(egui::Slider::new(&mut self.setting.luminance, 0.0..=1.0));
                    ui.end_row();

                    ui.label("亮度衰减");
                    ui.add(egui::Slider::new(
                        &mut self.setting.luminance_decay,
                        0.0..=1.0,
                    ));
                    ui.end_row();

                    ui.label("起始饱和度");
                    ui.add(egui::Slider::new(&mut self.setting.saturation, 0.0..=1.0));
                    ui.end_row();

                    ui.label("饱和度衰减");
                    ui.add(egui::Slider::new(
                        &mut self.setting.saturation_decay,
                        0.0..=1.0,
                    ));
                    ui.end_row();

                    ui.label("染色方式");
                    egui::ComboBox::from_id_salt("染色方式选择")
                        .selected_text(match self.setting.hue_mode {
                            HueMode::Fixed => "固定色相",
                            HueMode::Dynamic => "动态色相",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.setting.hue_mode,
                                HueMode::Fixed,
                                "固定色相",
                            );
                            ui.selectable_value(
                                &mut self.setting.hue_mode,
                                HueMode::Dynamic,
                                "动态色相",
                            );
                        });
                    ui.end_row();

                    match self.setting.hue_mode {
                        HueMode::Fixed => {
                            ui.label("色相1");
                            ui.add(
                                egui::Slider::new(
                                    &mut self.setting.hue1,
                                    0.0..=std::f32::consts::TAU,
                                )
                                .clamping(egui::SliderClamping::Never),
                            );
                            ui.end_row();

                            ui.label("色相2");
                            ui.add(
                                egui::Slider::new(
                                    &mut self.setting.hue2,
                                    0.0..=std::f32::consts::TAU,
                                )
                                .clamping(egui::SliderClamping::Never),
                            );
                            ui.end_row();
                        }
                        HueMode::Dynamic => {
                            for (i, target) in [
                                (1, &mut self.setting.hue_target1),
                                (2, &mut self.setting.hue_target2),
                                (3, &mut self.setting.hue_target3),
                            ] {
                                ui.label(format!("目标{i}"));
                                egui::ComboBox::from_id_salt(format!("目标{i}选择"))
                                    .selected_text(match *target {
                                        HueTarget::Omega1 => "ω1",
                                        HueTarget::Omega2 => "ω2",
                                        HueTarget::Omega3 => "ω3",
                                        HueTarget::Theta1 => "θ1",
                                        HueTarget::Theta2 => "θ2",
                                        HueTarget::Theta3 => "θ3",
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(target, HueTarget::Omega1, "ω1");
                                        ui.selectable_value(target, HueTarget::Omega2, "ω2");
                                        ui.selectable_value(target, HueTarget::Omega3, "ω3");
                                        if i == 3 {
                                            ui.selectable_value(target, HueTarget::Theta1, "θ1");
                                            ui.selectable_value(target, HueTarget::Theta2, "θ2");
                                            ui.selectable_value(target, HueTarget::Theta3, "θ3");
                                        } else {
                                            ui.selectable_value(target, HueTarget::Theta1, "θ1")
                                                .on_hover_text("⚠可能造成颜色突变");
                                            ui.selectable_value(target, HueTarget::Theta2, "θ2")
                                                .on_hover_text("⚠可能造成颜色突变");
                                            ui.selectable_value(target, HueTarget::Theta3, "θ3")
                                                .on_hover_text("⚠可能造成颜色突变");
                                        }
                                    });
                                ui.end_row();
                            }

                            ui.label("乘数");
                            ui.add(
                                egui::Slider::new(&mut self.setting.hue_factor, 0.01..=1.0)
                                    .clamping(egui::SliderClamping::Never),
                            );
                            ui.end_row();
                        }
                    }
                });
        });

        CollapsingHeader::new("debug info").show(ui, |ui| {
            egui::Grid::new("调试信息网格")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("状态");
                    ui.label(self.data.debug_message.clone());
                    ui.end_row();

                    ui.label("可见线段");
                    ui.label(format!(
                        "{}/{}",
                        self.data.line_count,
                        (2 << self.setting.depth) - 1
                    ));
                    ui.end_row();

                    ui.label("绘图耗时");
                    ui.label(format!("{}ms", self.data.frame_time));
                    ui.end_row();

                    ui.label("动能");
                    ui.label(self.data.t.to_string());
                    ui.end_row();

                    ui.label("势能");
                    ui.label(self.data.v.to_string());
                    ui.end_row();

                    ui.label("机械能")
                        .on_hover_text("应当守恒。若数值波动较大，说明求解异常。");
                    ui.label(self.data.e.to_string());
                    ui.end_row();
                });
        });

        if ui.button("重置").clicked() {
            *self = Self::default();
        }
    }
}

// -------- -------- -------- -------- -------- -------- -------- --------

// 数值解真好啊
type State = ode_solvers::Vector6<f64>;

struct Ode {
    g: f64,
    l: [f64; 3],
    m: [f64; 3],
}

impl Ode {
    fn new(a: &FractalPendulumApp) -> Self {
        Self {
            g: a.setting.g,
            l: a.setting.l,
            m: a.setting.m,
        }
    }
}

impl ode_solvers::System<f64, State> for Ode {
    fn system(&self, _: f64, y: &State, dy: &mut State) {
        let g = self.g;
        let [l1, l2, l3] = self.l;
        let [m1, m2, m3] = self.m;
        let q1 = y[0];
        let q2 = y[1];
        let q3 = y[2];
        let q4 = y[3];
        let q5 = y[4];
        let q6 = y[5];

        // sympy给我托梦来的
        let denominator = l1 * (m1 + m2 * q3.sin() * q3.sin() + m3 * q5.sin() * q5.sin());
        dy[0] = q2;
        dy[2] = q4;
        dy[4] = q6;
        dy[1] = (-g * l1 * (m1 + m2 + m3) * q1.sin()
            - g * l2 * m2 * (q1 + q3).sin()
            - g * l3 * m3 * (q1 + q5).sin()
            + g * l1 * m2 * (q1 + q3).sin() * q3.cos()
            + g * l1 * m3 * (q1 + q5).sin() * q5.cos()
            + l1 * l2 * m2 * q4 * q4 * q3.sin()
            + l1 * l3 * m3 * q6 * q6 * q5.sin())
            / l1
            / denominator;
        dy[3] = (-g * l1 * (m1 + m2) * q3.sin() * q1.cos()
            - g * l1 * m3 * (q1 + q3).sin() * q5.sin() * q5.sin()
            - g * l1 * m3 * (q1 + q5).sin() * q3.cos() * q5.cos()
            - l1 * l2 * m2 * q4 * q4 * q3.sin() * q3.cos()
            - l1 * l3 * m3 * q6 * q6 * q5.sin() * q3.cos()
            + g * l1 * m3 * q1.sin() * q3.cos()
            + g * l2 * m2 * (q1 + q3).sin() * q3.cos()
            + g * l3 * m3 * (q1 + q5).sin() * q3.cos())
            / l2
            / denominator;
        dy[5] = (-g * l1 * (m1 + m3) * q5.sin() * q1.cos()
            - g * l1 * m2 * (q1 + q3).sin() * q3.cos() * q5.cos()
            - g * l1 * m2 * (q1 + q5).sin() * q3.sin() * q3.sin()
            - l1 * l2 * m2 * q4 * q4 * q3.sin() * q5.cos()
            - l1 * l3 * m3 * q6 * q6 * q5.sin() * q5.cos()
            + g * l1 * m2 * q1.sin() * q5.cos()
            + g * l2 * m2 * (q1 + q3).sin() * q5.cos()
            + g * l3 * m3 * (q1 + q5).sin() * q5.cos())
            / l3
            / denominator;
    }
}

// -------- -------- -------- -------- -------- -------- -------- --------

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color32 {
    let h = if h >= 0.0 {
        h
    } else {
        h + std::f32::consts::TAU
    } / std::f32::consts::TAU
        * 360.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let (r, g, b) = (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    );
    Color32::from_rgb(r, g, b)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
