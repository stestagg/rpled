use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{execute, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::{Frame, Terminal};
use rpled_vm::fixture_parse;
use rpled_vm::sync::TokioSync;
use rpled_vm::vm::{HaltReason, NoVmDebug, VMError, VM};

const MEMORY_SIZE: usize = 4096;
const TICK_RATE: Duration = Duration::from_millis(75);

#[derive(Parser, Debug)]
#[command(version, about = "Interactive terminal debugger for rpled programs", long_about = None)]
struct Args {
    /// Path to the .pxs or .pxs.txt program fixture
    program: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Focus {
    Header,
    Stack,
    Heap,
    Disassembly,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Header => Focus::Stack,
            Focus::Stack => Focus::Heap,
            Focus::Heap => Focus::Disassembly,
            Focus::Disassembly => Focus::Header,
        }
    }
}

#[derive(Clone, Debug)]
struct DisasmRow {
    address: usize,
    opcode: u8,
    name: String,
}

struct App {
    vm: VM<MEMORY_SIZE, TokioSync, NoVmDebug>,
    disasm: Vec<DisasmRow>,
    disasm_index: usize,
    stack_scroll: usize,
    heap_scroll: usize,
    focus: Focus,
    breakpoints: HashSet<usize>,
    status: String,
    running: bool,
    last_tick: Instant,
}

impl App {
    async fn new(program: &[u8]) -> Result<Self, VMError> {
        let mut vm: VM<MEMORY_SIZE, TokioSync, NoVmDebug> = VM::new(NoVmDebug).await;
        vm.load(program)?;

        let disasm = build_disassembly(&vm.memory[..vm.heap_start]);

        Ok(Self {
            vm,
            disasm,
            disasm_index: 0,
            stack_scroll: 0,
            heap_scroll: 0,
            focus: Focus::Disassembly,
            breakpoints: HashSet::new(),
            status: "Loaded program".to_string(),
            running: false,
            last_tick: Instant::now(),
        })
    }

    async fn step_once(&mut self) {
        let pc_before = self.vm.pc;
        match self.vm.run_op().await {
            Ok(()) => {
                self.status = format!("Stepped from 0x{pc_before:04X} to 0x{:04X}", self.vm.pc);
            }
            Err(VMError::Halt(reason)) => {
                self.running = false;
                self.status = format!("Halted: {:?}", reason);
            }
            Err(err) => {
                self.running = false;
                self.status = format!("Error: {:?}", err);
            }
        }
    }

    async fn tick(&mut self) {
        if self.running && self.last_tick.elapsed() >= TICK_RATE {
            if self.breakpoints.contains(&self.vm.pc) {
                self.running = false;
                self.status = format!("Hit breakpoint at 0x{:04X}", self.vm.pc);
            } else {
                self.step_once().await;
            }
            self.last_tick = Instant::now();
        }
    }

    fn toggle_breakpoint(&mut self) {
        if let Some(row) = self.disasm.get(self.disasm_index) {
            if !self.breakpoints.insert(row.address) {
                self.breakpoints.remove(&row.address);
                self.status = format!("Removed breakpoint at 0x{:04X}", row.address);
            } else {
                self.status = format!("Added breakpoint at 0x{:04X}", row.address);
            }
        }
    }

    fn scroll_up(&mut self) {
        match self.focus {
            Focus::Stack => self.stack_scroll = self.stack_scroll.saturating_sub(1),
            Focus::Heap => self.heap_scroll = self.heap_scroll.saturating_sub(1),
            Focus::Disassembly => self.disasm_index = self.disasm_index.saturating_sub(1),
            Focus::Header => {}
        }
    }

    fn scroll_down(&mut self) {
        match self.focus {
            Focus::Stack => {
                let max = MEMORY_SIZE.saturating_sub(1);
                if self.stack_scroll < max {
                    self.stack_scroll += 1;
                }
            }
            Focus::Heap => {
                let max = self.vm.heap_end.saturating_sub(1);
                if self.heap_scroll < max {
                    self.heap_scroll += 1;
                }
            }
            Focus::Disassembly => {
                if self.disasm_index + 1 < self.disasm.len() {
                    self.disasm_index += 1;
                }
            }
            Focus::Header => {}
        }
    }
}

fn build_disassembly(program: &[u8]) -> Vec<DisasmRow> {
    let opcode_names = VM::<MEMORY_SIZE, TokioSync, NoVmDebug>::opcode_names();
    program
        .iter()
        .copied()
        .enumerate()
        .map(|(address, opcode)| {
            let name = opcode_names
                .iter()
                .find(|(op, _)| *op == opcode)
                .map(|(_, name)| (*name).to_string())
                .unwrap_or_else(|| "UNKNOWN".to_string());
            DisasmRow {
                address,
                opcode,
                name,
            }
        })
        .collect()
}

fn parse_program(path: &PathBuf) -> Result<Vec<u8>, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let bytes = if content.contains("=== OUTPUT ===") {
        fixture_parse::parse_fixture_with_output(&content).program
    } else {
        fixture_parse::decode_fixture(&content)
    };
    Ok(bytes)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let program = parse_program(&args.program)?;

    let mut app = App::new(&program)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e:?}")))?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, terminal::DisableLineWrap)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, terminal::EnableLineWrap)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("{err}");
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|frame| draw_ui(frame, app))?;

        app.tick().await;

        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('s') => app.step_once().await,
                        KeyCode::Char('r') => {
                            app.running = true;
                            app.status = "Running".to_string();
                            app.last_tick = Instant::now();
                        }
                        KeyCode::Char('p') => {
                            app.running = false;
                            app.status = "Paused".to_string();
                        }
                        KeyCode::Char('b') => app.toggle_breakpoint(),
                        KeyCode::Tab => app.focus = app.focus.next(),
                        KeyCode::Up => app.scroll_up(),
                        KeyCode::Down => app.scroll_down(),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn draw_ui(frame: &mut Frame, app: &mut App) {
    let size = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(size);

    draw_header(frame, chunks[0], app);
    draw_body(frame, chunks[1], app);
    draw_status(frame, chunks[2], app);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let header_text = format!(
        "PC: 0x{pc:04X}   SP: 0x{sp:04X}   Heap: 0x{heap_start:04X} - 0x{heap_end:04X}   Max PC: 0x{max_pc:04X}",
        pc = app.vm.pc,
        sp = app.vm.sp,
        heap_start = app.vm.heap_start,
        heap_end = app.vm.heap_end,
        max_pc = app.vm.max_pc,
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Header")
        .border_style(focus_style(Focus::Header, app.focus));

    let paragraph = Paragraph::new(header_text).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(40),
        ])
        .split(area);

    draw_stack(frame, columns[0], app);
    draw_heap(frame, columns[1], app);
    draw_disassembly(frame, columns[2], app);
}

fn focus_style(target: Focus, focus: Focus) -> Style {
    if target == focus {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

fn draw_stack(frame: &mut Frame, area: Rect, app: &App) {
    let mut items = Vec::new();
    for (idx, chunk) in app
        .vm
        .memory
        .iter()
        .enumerate()
        .skip(app.vm.sp + app.stack_scroll)
        .take(area.height as usize)
    {
        let item = ListItem::new(format!(
            "0x{addr:04X}: 0x{value:02X}",
            addr = idx,
            value = chunk
        ));
        items.push(item);
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Stack")
                .border_style(focus_style(Focus::Stack, app.focus)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_widget(list, area);
}

fn draw_heap(frame: &mut Frame, area: Rect, app: &App) {
    let mut items = Vec::new();
    if app.vm.heap_start < app.vm.heap_end {
        for (offset, value) in app
            .vm
            .memory
            .iter()
            .enumerate()
            .skip(app.vm.heap_start + app.heap_scroll)
            .take(area.height as usize)
        {
            let item = ListItem::new(format!(
                "0x{addr:04X}: 0x{value:02X}",
                addr = offset,
                value = value
            ));
            items.push(item);
        }
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Heap")
                .border_style(focus_style(Focus::Heap, app.focus)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_widget(list, area);
}

fn draw_disassembly(frame: &mut Frame, area: Rect, app: &App) {
    let mut items = Vec::new();
    let mut start_index = app.disasm_index.saturating_sub(area.height as usize / 2);
    if start_index + area.height as usize > app.disasm.len() {
        start_index = app.disasm.len().saturating_sub(area.height as usize);
    }

    for row in app
        .disasm
        .iter()
        .skip(start_index)
        .take(area.height as usize)
    {
        let mut line = format!(
            "0x{addr:04X}: {opcode:02X} {name}",
            addr = row.address,
            opcode = row.opcode,
            name = row.name
        );
        if app.breakpoints.contains(&row.address) {
            line.push_str("  [brk]");
        }
        let item = ListItem::new(line);
        items.push(item);
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Disassembly")
                .border_style(focus_style(Focus::Disassembly, app.focus)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("→ ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.disasm_index.saturating_sub(start_index)));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let text = format!(
        "Status: {status} | Commands: [s]tep [r]un [p]ause [b]reakpoint [Tab] switch pane [q]uit",
        status = app.status
    );
    let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

#[allow(dead_code)]
fn describe_halt(reason: HaltReason) -> &'static str {
    match reason {
        HaltReason::Signal => "Signal",
        HaltReason::HaltOp => "HALT opcode",
        HaltReason::ProgramEnd => "Program end",
    }
}
