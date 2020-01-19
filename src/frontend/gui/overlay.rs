use crate::frontend::gui::termwindow::TermWindow;
use crate::mux::tab::{Tab, TabId};
use crate::termwiztermtab::{allocate, TermWizTerminal, TermWizTerminalTab};
use std::pin::Pin;
use std::rc::Rc;
use termwiz::lineedit::*;
use termwiz::surface::{Change, SequenceNo, Surface};
use termwiz::terminal::{ScreenSize, Terminal, TerminalWaker};
use window::Window;

pub fn start_overlay<T, F>(
    term_window: &TermWindow,
    tab: &Rc<dyn Tab>,
    func: F,
) -> (
    Rc<dyn Tab>,
    Pin<Box<dyn std::future::Future<Output = Option<anyhow::Result<T>>>>>,
)
where
    T: Send + 'static,
    F: Send + 'static + FnOnce(TabId, TermWizTerminal) -> anyhow::Result<T>,
{
    let tab_id = tab.tab_id();
    let dims = tab.renderer().get_dimensions();
    let (tw_term, tw_tab) = allocate(dims.cols, dims.viewport_rows);

    let window = term_window.window.clone().unwrap();

    let future = promise::spawn::spawn_into_new_thread(move || {
        let res = func(tab_id, tw_term);
        TermWindow::schedule_cancel_overlay(window, tab_id);
        res
    });

    (Rc::new(tw_tab), Box::pin(future))
}

pub fn tab_navigator(tab_id: TabId, mut term: TermWizTerminal) -> anyhow::Result<()> {
    term.render(&[
        Change::Title("Tab Navigator".to_string()),
        Change::Text("Navigate!\r\n".to_string()),
    ])?;

    let mut editor = LineEditor::new(term);
    editor.set_prompt("(press enter to return to your tab)");

    let mut host = NopLineEditorHost::default();
    editor.read_line(&mut host).ok();

    Ok(())
}