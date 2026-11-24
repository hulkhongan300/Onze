use crate::formations::{COLS, FORMATIONS, ROWS};
use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Key, KeyboardShortcut, Modifiers, PointerButton,
    Popup, Pos2, Rect, Sense, SetOpenCommand, Shape, Stroke, StrokeKind, Vec2,
    emath::RectTransform, pos2, vec2,
};
use serde::{Deserialize, Serialize};

/// Real pitch size in metres, used for the markings.
const W: f32 = 68.0;
const L: f32 = 105.0;
const MARGIN: f32 = 3.0;
/// Screen-space sizes, in pixels.
const ERASER_PX: f32 = 12.0;
const PEN_STEP_PX: f32 = 2.0;

const TEAM: Color32 = Color32::from_rgb(198, 40, 40);
const KEEPER: Color32 = Color32::from_rgb(249, 168, 37);
const SELECTION: Color32 = Color32::from_rgb(30, 136, 229);
const PEN_COLORS: [(&str, Color32); 5] = [
    ("Black", Color32::from_rgb(33, 33, 33)),
    ("Red", Color32::from_rgb(229, 57, 53)),
    ("Blue", Color32::from_rgb(30, 136, 229)),
    ("Orange", Color32::from_rgb(251, 140, 0)),
    ("Purple", Color32::from_rgb(142, 36, 170)),
];

/// Board positions are in grid units (x in 0..COLS, y in 0..ROWS), so players and drawings
/// stay put relative to each other when switching between half and full field.
fn cell_center((c, r): (u8, u8)) -> Pos2 {
    pos2(c as f32 + 0.5, r as f32 + 0.5)
}

fn snap(p: Pos2) -> Pos2 {
    pos2(
        p.x.floor().clamp(0.0, COLS as f32 - 1.0) + 0.5,
        p.y.floor().clamp(0.0, ROWS as f32 - 1.0) + 0.5,
    )
}

/// Clamps the move `d` so every player in `who` stays at least `inset` inside the grid.
/// One shared clamp keeps a group's shape intact at the edges.
fn clamp_delta(mut d: Vec2, start: &[Pos2; 11], who: &[usize], inset: f32) -> Vec2 {
    for &i in who {
        let s = start[i];
        d.x = d.x.clamp(inset - s.x, COLS as f32 - inset - s.x);
        d.y = d.y.clamp(inset - s.y, ROWS as f32 - inset - s.y);
    }
    d
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum Surface {
    White,
    Grass,
}

impl Surface {
    fn stripes(self) -> [Color32; 2] {
        match self {
            Surface::White => [Color32::from_gray(0xEB), Color32::from_gray(0xC0)],
            Surface::Grass => [
                Color32::from_rgb(46, 125, 50),
                Color32::from_rgb(56, 142, 60),
            ],
        }
    }

    /// Colour of the pitch markings.
    fn ink(self) -> Color32 {
        match self {
            Surface::White => Color32::from_gray(80),
            Surface::Grass => Color32::WHITE,
        }
    }
}

/// What each player token shows.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum Labels {
    Number,
    Name,
    Both,
    None,
}

/// Shirt number and name, kept per player index so they survive formation changes.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Player {
    number: u8,
    name: String,
}

impl Labels {
    /// Text inside the circle and text below it. Name mode falls back to the number.
    fn texts(self, p: &Player) -> (Option<String>, Option<&str>) {
        let name = Some(p.name.trim()).filter(|n| !n.is_empty());
        let number = Some(p.number.to_string());
        match self {
            Labels::Number => (number, None),
            Labels::Name if name.is_some() => (None, name),
            Labels::Name => (number, None),
            Labels::Both => (number, name),
            Labels::None => (None, None),
        }
    }
}

#[derive(Clone, PartialEq)]
struct Line {
    points: Vec<Pos2>,
    color: Color32,
    width: f32,
}

/// Everything undo restores.
#[derive(Clone, PartialEq)]
struct Board {
    formation: usize,
    players: [Pos2; 11],
    /// Where each player's dashed movement line starts: the formation spot, or the saved spot.
    home: [Pos2; 11],
    lines: Vec<Line>,
}

impl Board {
    fn set_formation(&mut self, i: usize) {
        self.formation = i;
        self.players = FORMATIONS[i].slots.map(cell_center);
        self.home = self.players;
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Move,
    Pen,
    Eraser,
}

pub struct Onze {
    board: Board,
    // shortcut: unbounded full-board snapshots, fine for now; cap it if sessions get long.
    history: Vec<Board>,
    tool: Tool,
    color: Color32,
    width: f32,
    surface: Surface,
    full_field: bool,
    // Not on Board: typing names is not an undo step.
    team: [Player; 11],
    labels: Labels,
    show_grid: bool,
    show_panel: bool,
    selected: [bool; 11],
    grab: Option<Grab>,
    /// Screen point where the selection box drag started.
    lasso: Option<Pos2>,
}

/// What survives a refresh or restart. Drawings and undo history are left out.
/// `serde(default)` lets data saved by an older version fill in only the fields it has.
#[derive(PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Saved {
    team: [Player; 11],
    labels: Labels,
    formation: usize,
    players: [Pos2; 11],
    home: [Pos2; 11],
    surface: Surface,
    full_field: bool,
    color: Color32,
}

impl Default for Saved {
    fn default() -> Self {
        Onze::default().saved()
    }
}

/// Players being dragged, with where they and the pointer started (grid units).
struct Grab {
    origin: Pos2,
    start: [Pos2; 11],
    who: Vec<usize>,
}

impl Default for Onze {
    fn default() -> Self {
        let mut board = Board {
            formation: 0,
            players: [Pos2::ZERO; 11],
            home: [Pos2::ZERO; 11],
            lines: vec![],
        };
        board.set_formation(0);
        Self {
            board,
            history: vec![],
            tool: Tool::Move,
            color: PEN_COLORS[0].1,
            width: 3.0,
            surface: Surface::White,
            full_field: false,
            team: std::array::from_fn(|i| Player {
                number: i as u8 + 1,
                name: String::new(),
            }),
            labels: Labels::Number,
            show_grid: false,
            show_panel: true,
            selected: [false; 11],
            grab: None,
            lasso: None,
        }
    }
}

impl Onze {
    /// Restores the last session's state, or the defaults if there is none or it doesn't decode.
    pub fn new(cc: &eframe::CreationContext) -> Self {
        cc.storage
            .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
            .map_or_else(Self::default, Self::from_saved)
    }

    fn from_saved(s: Saved) -> Self {
        let mut app = Self::default();
        // A stale index (formation list changed) or off-grid spots keep the default board.
        let on_grid =
            |p: &Pos2| (0.0..=COLS as f32).contains(&p.x) && (0.0..=ROWS as f32).contains(&p.y);
        if s.formation < FORMATIONS.len() {
            app.board.set_formation(s.formation);
            if s.players.iter().chain(&s.home).all(on_grid) {
                app.board.players = s.players;
                app.board.home = s.home;
            }
        }
        app.team = s.team.map(|p| Player {
            number: p.number.min(99),
            ..p
        });
        app.labels = s.labels;
        app.surface = s.surface;
        app.full_field = s.full_field;
        app.color = s.color;
        app
    }

    fn saved(&self) -> Saved {
        Saved {
            team: self.team.clone(),
            labels: self.labels,
            formation: self.board.formation,
            players: self.board.players,
            home: self.board.home,
            surface: self.surface,
            full_field: self.full_field,
            color: self.color,
        }
    }
}

/// The quick-pick colour row. Returns true when a colour was picked.
fn swatches(ui: &mut egui::Ui, color: &mut Color32) -> bool {
    let mut picked = false;
    ui.horizontal(|ui| {
        for (name, c) in PEN_COLORS {
            let ring = if *color == c {
                Stroke::new(2.0, ui.visuals().strong_text_color())
            } else {
                Stroke::NONE
            };
            let swatch = egui::Button::new("")
                .fill(c)
                .stroke(ring)
                .min_size(vec2(20.0, 20.0));
            if ui.add(swatch).on_hover_text(name).clicked() {
                *color = c;
                picked = true;
            }
        }
    });
    picked
}

impl Onze {
    /// Applies `f` and records an undo step only if the board actually changed.
    fn edit(&mut self, f: impl FnOnce(&mut Board)) {
        let before = self.board.clone();
        f(&mut self.board);
        if self.board != before {
            self.history.push(before);
        }
    }

    fn undo(&mut self) {
        if let Some(b) = self.history.pop() {
            self.board = b;
        }
    }

    /// Closest player whose token contains the screen point `p`.
    fn player_at(&self, p: Pos2, to_screen: &RectTransform, r: f32) -> Option<usize> {
        let d = |i: usize| (*to_screen * self.board.players[i]).distance(p);
        (0..11)
            .filter(|&i| d(i) <= r)
            .min_by(|&a, &b| d(a).total_cmp(&d(b)))
    }

    /// Shared by the side panel and the click menu.
    fn tool_controls(&mut self, ui: &mut egui::Ui) {
        ui.selectable_value(&mut self.tool, Tool::Move, "Move players");
        ui.selectable_value(&mut self.tool, Tool::Pen, "Pen");
        ui.selectable_value(&mut self.tool, Tool::Eraser, "Eraser");
        if swatches(ui, &mut self.color) {
            self.tool = Tool::Pen;
        }
    }

    /// Shared by the side panel and the click menu.
    fn edit_buttons(&mut self, ui: &mut egui::Ui) {
        if ui
            .add_enabled(!self.history.is_empty(), egui::Button::new("Undo (Ctrl+Z)"))
            .clicked()
        {
            self.undo();
        }
        if ui.button("Clear drawings").clicked() {
            self.edit(|b| b.lines.clear());
        }
        if ui.button("Reset players").clicked() {
            self.edit(|b| b.set_formation(b.formation));
        }
        if ui
            .add_enabled(
                self.board.players != self.board.home,
                egui::Button::new("Save positions"),
            )
            .on_hover_text("Make these the players' new spots")
            .on_disabled_hover_text("No unsaved moves")
            .clicked()
        {
            self.edit(|b| b.home = b.players);
        }
    }

    fn panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Onze");
        ui.separator();

        let mut f = self.board.formation;
        egui::ComboBox::from_label("Formation")
            .selected_text(FORMATIONS[f].name)
            .show_ui(ui, |ui| {
                for (i, form) in FORMATIONS.iter().enumerate() {
                    ui.selectable_value(&mut f, i, form.name);
                }
            });
        if f != self.board.formation {
            self.edit(|b| b.set_formation(f));
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Pitch");
            ui.selectable_value(&mut self.surface, Surface::White, "White");
            ui.selectable_value(&mut self.surface, Surface::Grass, "Grass");
        });
        ui.checkbox(&mut self.full_field, "Full field");
        ui.checkbox(&mut self.show_grid, "Show grid");
        ui.horizontal(|ui| {
            ui.label("Labels");
            ui.selectable_value(&mut self.labels, Labels::Number, "Number");
            ui.selectable_value(&mut self.labels, Labels::Name, "Name");
            ui.selectable_value(&mut self.labels, Labels::Both, "Both");
            ui.selectable_value(&mut self.labels, Labels::None, "None");
        });

        ui.separator();
        ui.label("Team");
        for p in &mut self.team {
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut p.number).range(0..=99));
                ui.add(
                    egui::TextEdit::singleline(&mut p.name)
                        .hint_text("Name")
                        .desired_width(120.0),
                );
            });
        }

        ui.separator();
        ui.label("Tool");
        self.tool_controls(ui);
        ui.horizontal(|ui| {
            ui.label("Custom");
            ui.color_edit_button_srgba(&mut self.color);
        });
        ui.add(egui::Slider::new(&mut self.width, 1.0..=10.0).text("width"));

        ui.separator();
        self.edit_buttons(ui);

        ui.separator();
        ui.weak("H: hide panel\nRight-click pitch: quick menu\nMove: drag a box or click players\nto select, drag one to move the group");
    }

    fn pitch(&mut self, ui: &mut egui::Ui) {
        let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        // Visible part of the pitch in metres; the team grid fills it (bottom half when half field).
        let y0 = if self.full_field { 0.0 } else { L / 2.0 };
        let top_margin = if self.full_field { MARGIN } else { 0.5 };
        let view = Rect::from_min_max(pos2(-MARGIN, y0 - top_margin), pos2(W + MARGIN, L + MARGIN));
        let scale = (resp.rect.width() / view.width()).min(resp.rect.height() / view.height());
        let view_screen = Rect::from_center_size(resp.rect.center(), view.size() * scale);
        let m2s = RectTransform::from_to(view, view_screen);
        let grid = Rect::from_min_size(Pos2::ZERO, vec2(COLS as f32, ROWS as f32));
        let g2s = RectTransform::from_to(
            grid,
            m2s.transform_rect(Rect::from_min_max(pos2(0.0, y0), pos2(W, L))),
        );
        let to_grid = |p: Pos2| grid.clamp(g2s.inverse() * p);
        let r = 0.4 * g2s.scale().min_elem();

        // --- input: drags use the tool, a right-click opens the quick menu ---
        if resp.drag_started_by(PointerButton::Primary) {
            self.history.push(self.board.clone());
            let origin = ui.input(|i| i.pointer.press_origin()).unwrap_or_default();
            match self.tool {
                Tool::Move => {
                    let hit = self.player_at(origin, &g2s, r);
                    let who: Vec<usize> = match hit {
                        Some(i) => {
                            if !self.selected[i] {
                                self.selected = [false; 11];
                                self.selected[i] = true;
                            }
                            (0..11).filter(|&j| self.selected[j]).collect()
                        }
                        None => {
                            self.lasso = Some(origin);
                            vec![]
                        }
                    };
                    if !who.is_empty() {
                        self.grab = Some(Grab {
                            origin: to_grid(origin),
                            start: self.board.players,
                            who,
                        });
                    }
                }
                Tool::Pen => self.board.lines.push(Line {
                    points: vec![to_grid(origin)],
                    color: self.color,
                    width: self.width,
                }),
                Tool::Eraser => {}
            }
        }
        if resp.dragged_by(PointerButton::Primary)
            && let Some(p) = resp.interact_pointer_pos()
        {
            match self.tool {
                Tool::Move => {
                    if let Some(g) = &self.grab {
                        let d = clamp_delta(to_grid(p) - g.origin, &g.start, &g.who, 0.0);
                        for &i in &g.who {
                            self.board.players[i] = g.start[i] + d;
                        }
                    } else if let Some(a) = self.lasso {
                        let area = Rect::from_two_pos(a, p);
                        self.selected =
                            std::array::from_fn(|i| area.contains(g2s * self.board.players[i]));
                    }
                }
                Tool::Pen => {
                    if let Some(line) = self.board.lines.last_mut()
                        && line
                            .points
                            .last()
                            .is_none_or(|&q| (g2s * q).distance(p) > PEN_STEP_PX)
                    {
                        line.points.push(to_grid(p));
                    }
                }
                Tool::Eraser => self
                    .board
                    .lines
                    .retain(|l| l.points.iter().all(|&q| (g2s * q).distance(p) > ERASER_PX)),
            }
        }
        if resp.drag_stopped_by(PointerButton::Primary) {
            match self.tool {
                Tool::Move => {
                    if let Some(g) = self.grab.take() {
                        // Whole-cell move for the group, so everyone snaps by the same amount.
                        let moved = self.board.players[g.who[0]] - g.start[g.who[0]];
                        let d = clamp_delta(moved.round(), &g.start, &g.who, 0.5);
                        for &i in &g.who {
                            self.board.players[i] = snap(g.start[i] + d);
                        }
                    }
                    self.lasso = None;
                }
                Tool::Pen => {
                    if self.board.lines.last().is_some_and(|l| l.points.len() < 2) {
                        self.board.lines.pop();
                    }
                }
                Tool::Eraser => {}
            }
            if self.history.last() == Some(&self.board) {
                self.history.pop();
            }
        }
        if self.tool == Tool::Move
            && resp.clicked()
            && let Some(p) = resp.interact_pointer_pos()
        {
            match self.player_at(p, &g2s, r) {
                Some(i) => self.selected[i] = !self.selected[i],
                None => self.selected = [false; 11],
            }
        }
        let menu = Popup::menu(&resp).at_pointer_fixed();
        // Bool(true), not Toggle: only Bool records the pointer position the menu opens at.
        let set = resp
            .secondary_clicked()
            .then(|| SetOpenCommand::Bool(!menu.is_open()));
        menu.open_memory(set).show(|ui| {
            self.tool_controls(ui);
            ui.separator();
            self.edit_buttons(ui);
        });
        if self.tool == Tool::Move
            && let Some(p) = resp.hover_pos()
            && (self.grab.is_some() || self.player_at(p, &g2s, r).is_some())
        {
            ui.ctx().set_cursor_icon(if self.grab.is_some() {
                CursorIcon::Grabbing
            } else {
                CursorIcon::Grab
            });
        }

        // --- drawing: pitch, players, then drawings on top ---
        let [stripe_a, stripe_b] = self.surface.stripes();
        let ink = self.surface.ink();
        painter.rect_filled(view_screen, 0.0, stripe_a);
        for row in 0..ROWS {
            let band =
                Rect::from_min_max(pos2(0.0, row as f32), pos2(COLS as f32, row as f32 + 1.0));
            painter.rect_filled(
                g2s.transform_rect(band),
                0.0,
                if row % 2 == 0 { stripe_a } else { stripe_b },
            );
        }
        if self.show_grid {
            let faint = Stroke::new(1.0, ink.gamma_multiply(0.25));
            for c in 1..COLS {
                painter.line_segment(
                    [g2s * pos2(c as f32, 0.0), g2s * pos2(c as f32, ROWS as f32)],
                    faint,
                );
            }
            for row in 1..ROWS {
                painter.line_segment(
                    [
                        g2s * pos2(0.0, row as f32),
                        g2s * pos2(COLS as f32, row as f32),
                    ],
                    faint,
                );
            }
        }

        // Markings in metres, clipped so the half view cuts cleanly at the halfway line.
        let marks = painter.with_clip_rect(view_screen);
        let at = |x: f32, y: f32| m2s * pos2(x, y);
        let rect = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_two_pos(at(x0, y0), at(x1, y1));
        let line = Stroke::new(2.0, ink);
        marks.rect_stroke(rect(0.0, 0.0, W, L), 0.0, line, StrokeKind::Middle);
        marks.line_segment([at(0.0, L / 2.0), at(W, L / 2.0)], line);
        marks.circle_stroke(at(W / 2.0, L / 2.0), 9.15 * scale, line);
        marks.circle_filled(at(W / 2.0, L / 2.0), 0.4 * scale, ink);
        for (goal, dir) in [(0.0, 1.0), (L, -1.0)] {
            let box_ =
                |w: f32, d: f32| rect(W / 2.0 - w / 2.0, goal, W / 2.0 + w / 2.0, goal + dir * d);
            marks.rect_stroke(box_(40.32, 16.5), 0.0, line, StrokeKind::Middle);
            marks.rect_stroke(box_(18.32, 5.5), 0.0, line, StrokeKind::Middle);
            marks.rect_stroke(box_(7.32, -2.0), 0.0, line, StrokeKind::Middle);
            let spot = pos2(W / 2.0, goal + dir * 11.0);
            marks.circle_filled(m2s * spot, 0.4 * scale, ink);
            // Penalty arc: the part of the 9.15 m circle around the spot that lies outside the box.
            let half = (5.5f32 / 9.15).acos();
            let arc = (0..=16)
                .map(|k| -half + 2.0 * half * k as f32 / 16.0)
                .map(|a| m2s * (spot + vec2(a.sin(), dir * a.cos()) * 9.15))
                .collect();
            marks.add(Shape::line(arc, line));
        }

        let ghost = Stroke::new(1.5, ink.gamma_multiply(0.6));
        for (&p, &home) in self.board.players.iter().zip(&self.board.home) {
            if p != home {
                painter.circle_stroke(g2s * home, r, ghost);
                painter.extend(Shape::dashed_line(&[g2s * home, g2s * p], ghost, 6.0, 4.0));
            }
        }
        for (i, &p) in self.board.players.iter().enumerate() {
            let c = g2s * p;
            painter.circle_filled(c, r, if i == 0 { KEEPER } else { TEAM });
            painter.circle_stroke(c, r, Stroke::new(2.0, Color32::WHITE));
            if self.tool == Tool::Move && self.selected[i] {
                painter.circle_stroke(c, r + 3.0, Stroke::new(2.5, SELECTION));
            }
            let (inside, below) = self.labels.texts(&self.team[i]);
            if let Some(t) = inside {
                painter.text(
                    c,
                    Align2::CENTER_CENTER,
                    t,
                    FontId::proportional(r),
                    Color32::WHITE,
                );
            }
            if let Some(t) = below {
                painter.text(
                    c + vec2(0.0, r + 2.0),
                    Align2::CENTER_TOP,
                    t,
                    FontId::proportional(0.8 * r),
                    ink,
                );
            }
        }

        for l in &self.board.lines {
            let pts = l.points.iter().map(|&p| g2s * p).collect();
            painter.add(Shape::line(pts, Stroke::new(l.width, l.color)));
        }

        if let (Some(a), Some(p)) = (self.lasso, resp.interact_pointer_pos()) {
            let area = Rect::from_two_pos(a, p);
            painter.rect_filled(area, 0.0, SELECTION.gamma_multiply(0.15));
            painter.rect_stroke(area, 0.0, Stroke::new(1.0, SELECTION), StrokeKind::Inside);
        }
    }
}

impl eframe::App for Onze {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.saved());
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // While typing a name, Ctrl+Z belongs to the text field.
        if !ctx.egui_wants_keyboard_input()
            && ctx.input_mut(|i| {
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Z))
            })
        {
            self.undo();
        }
        if !ctx.egui_wants_keyboard_input()
            && ctx.input(|i| i.modifiers.is_none() && i.key_pressed(Key::H))
        {
            self.show_panel = !self.show_panel;
        }
        if self.show_panel {
            egui::Panel::left("tools").show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.panel(ui))
            });
        }
        egui::CentralPanel::default().show(ui, |ui| self.pitch(ui));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_goes_to_cell_center_and_stays_on_grid() {
        assert_eq!(
            snap(cell_center((3, 7)) + vec2(0.3, -0.4)),
            cell_center((3, 7))
        );
        assert_eq!(snap(pos2(-5.0, 500.0)), cell_center((0, ROWS - 1)));
        assert_eq!(snap(pos2(COLS as f32, 0.0)), cell_center((COLS - 1, 0)));
    }

    #[test]
    fn group_move_keeps_shape_at_the_edge() {
        let mut start = [cell_center((0, 0)); 11];
        start[1] = cell_center((7, 5));
        start[2] = cell_center((8, 5));
        // Dragged 3 cells right: the player already on the last column stops the whole group.
        assert_eq!(
            clamp_delta(vec2(3.0, 0.0), &start, &[1, 2], 0.5),
            vec2(0.0, 0.0)
        );
        assert_eq!(
            clamp_delta(vec2(-2.0, 1.0), &start, &[1, 2], 0.5),
            vec2(-2.0, 1.0)
        );
    }

    #[test]
    fn save_makes_moved_players_the_new_home() {
        let mut app = Onze::default();
        app.edit(|b| b.players[5] = cell_center((0, 0)));
        assert_ne!(app.board.players, app.board.home);
        app.edit(|b| b.home = b.players);
        assert_eq!(app.board.players, app.board.home);
        app.undo();
        assert_ne!(app.board.players, app.board.home);
    }

    #[test]
    fn labels_fall_back_to_number_without_a_name() {
        let mut p = Player {
            number: 9,
            name: " ".into(),
        };
        assert_eq!(Labels::Name.texts(&p), (Some("9".into()), None));
        assert_eq!(Labels::Both.texts(&p), (Some("9".into()), None));
        p.name = "Eusébio".into();
        assert_eq!(Labels::Name.texts(&p), (None, Some("Eusébio")));
        assert_eq!(Labels::Both.texts(&p), (Some("9".into()), Some("Eusébio")));
        assert_eq!(Labels::None.texts(&p), (None, None));
    }

    #[test]
    fn team_survives_formation_change() {
        let mut app = Onze::default();
        app.team[3].name = "Ruben".into();
        app.edit(|b| b.set_formation(2));
        app.edit(|b| b.set_formation(b.formation));
        assert_eq!(app.team[3].name, "Ruben");
        assert_eq!(app.team[10].number, 11);
    }

    #[derive(Default)]
    struct Mem(std::collections::HashMap<String, String>);

    impl eframe::Storage for Mem {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.0.insert(key.into(), value);
        }
        fn remove_string(&mut self, key: &str) {
            self.0.remove(key);
        }
        fn flush(&mut self) {}
    }

    fn load(mem: &Mem) -> Option<Saved> {
        eframe::get_value(mem, eframe::APP_KEY)
    }

    #[test]
    fn state_round_trips_through_storage() {
        let mut app = Onze::default();
        app.team[3].name = "Ruben".into();
        app.team[9].number = 77;
        app.labels = Labels::Both;
        app.surface = Surface::Grass;
        app.full_field = true;
        app.color = PEN_COLORS[2].1;
        app.edit(|b| b.set_formation(2));
        app.edit(|b| b.players[5] = cell_center((0, 0)));
        app.edit(|b| b.home = b.players);
        let mut mem = Mem::default();
        eframe::App::save(&mut app, &mut mem);
        let back = Onze::from_saved(load(&mem).unwrap());
        assert!(back.saved() == app.saved());
    }

    #[test]
    fn bad_stored_state_falls_back_to_defaults() {
        let mut s = Saved::default();
        s.team[3].name = "Ruben".into();
        s.formation = 999;
        let app = Onze::from_saved(s);
        assert_eq!(app.board.formation, 0);
        assert!(app.board.players == Onze::default().board.players);
        assert_eq!(app.team[3].name, "Ruben");

        let mut s = Saved::default();
        s.players[0] = pos2(f32::NAN, 500.0);
        assert!(Onze::from_saved(s).board.players == Onze::default().board.players);

        let mut mem = Mem::default();
        mem.0.insert(eframe::APP_KEY.into(), "not ron".into());
        assert!(load(&mem).is_none());
        // Older data with fewer fields keeps what it has and defaults the rest.
        mem.0
            .insert(eframe::APP_KEY.into(), "(labels: Name)".into());
        let s = load(&mem).unwrap();
        assert!(s.labels == Labels::Name);
        assert_eq!(s.team[0].number, 1);
    }

    #[test]
    fn edit_records_undo_only_on_change() {
        let mut app = Onze::default();
        app.edit(|b| b.set_formation(0));
        assert!(app.history.is_empty());
        app.edit(|b| b.set_formation(2));
        app.undo();
        assert_eq!(app.board.formation, 0);
    }
}
