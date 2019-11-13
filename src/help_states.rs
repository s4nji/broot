use std::io::Write;

use crossterm::{
    queue,
    terminal::{Clear, ClearType},
};
use termimad::{Area, MadView};

use crate::{
    app_state::{AppState, AppStateCmdResult},
    app_context::AppContext,
    commands::{Action, Command},
    conf::Conf,
    errors::ProgramError,
    help_content,
    io::W,
    mad_skin,
    screens::Screen,
    status::Status,
    task_sync::TaskLifetime,
    verb_store::PrefixSearchResult,
    verbs::VerbExecutor,
};

/// an application state dedicated to help
pub struct HelpState {
    dirty: bool, // when the screen background must be cleared
    pub view: MadView,
}

impl HelpState {
    pub fn new(screen: &Screen, con: &AppContext) -> HelpState {
        let area = Area::uninitialized(); // will be fixed at drawing time
        let markdown = help_content::build_markdown(con);
        let view = MadView::from(markdown, area, mad_skin::make_help_mad_skin(&screen.skin));
        HelpState {
            dirty: true,
            view,
        }
    }

    fn resize_area(&mut self, screen: &Screen) {
        let mut area = Area::new(0, 0, screen.width, screen.height - 2);
        area.pad_for_max_width(110);
        self.view.resize(&area);
    }
}

impl AppState for HelpState {

    fn has_pending_task(&self) -> bool {
        false
    }

    fn apply(
        &mut self,
        cmd: &mut Command,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError> {
        self.resize_area(screen);
        Ok(match &cmd.action {
            Action::Back => AppStateCmdResult::PopState,
            Action::VerbIndex(index) => {
                let verb = &con.verb_store.verbs[*index];
                self.execute_verb(verb, &verb.invocation, screen, con)?
            }
            Action::VerbInvocate(invocation) => match con.verb_store.search(&invocation.key) {
                PrefixSearchResult::Match(verb) => {
                    self.execute_verb(verb, &invocation, screen, con)?
                }
                _ => AppStateCmdResult::verb_not_found(&invocation.key),
            },
            Action::MoveSelection(dy) => {
                self.view.try_scroll_lines(*dy);
                AppStateCmdResult::Keep
            }
            _ => AppStateCmdResult::Keep,
        })
    }

    fn refresh(&mut self, _screen: &Screen, _con: &AppContext) -> Command {
        Command::new()
    }

    fn do_pending_task(&mut self, _screen: &mut Screen, _tl: &TaskLifetime) {
        unreachable!();
    }

    fn display(
        &mut self,
        w: &mut W,
        screen: &Screen,
        _con: &AppContext
    ) -> Result<(), ProgramError> {
        if self.dirty {
            // we don't clear the whole screen more than necessary because
            //  it makes scrolling flicker
            screen.clear(w)?;
            self.dirty = false;
        }
        self.resize_area(screen);
        Ok(self.view.write_on(w)?)
    }

    fn write_status(
        &self,
        w: &mut W,
        cmd: &Command,
        screen: &Screen,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        match &cmd.action {
            Action::VerbEdit(invocation) => match con.verb_store.search(&invocation.key) {
                PrefixSearchResult::NoMatch => {
                    Status::from_error(mad_inline!("No matching verb")).display(w, screen)
                }
                PrefixSearchResult::Match(verb) => {
                    verb.write_status(w, None, Conf::default_location(), invocation, screen)
                }
                PrefixSearchResult::TooManyMatches => Status::from_message(mad_inline!(
                    "Type a verb then *enter* to execute it"
                )).display(w, screen),
            }
            _ => Status::from_message(mad_inline!(
                "Hit *esc* to get back to the tree, or a space to start a verb"
            )).display(w, screen),
        }
    }

    /// there's no meaningful flags here
    fn write_flags(
        &self,
        w: &mut W,
        screen: &mut Screen,
        _con: &AppContext
    ) -> Result<(), ProgramError> {
        screen.skin.default.queue_bg(w)?;
        queue!(w, Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
}
