use std::collections::HashSet;

use gamemanager_core::{GameSummary, GameViewMode};

#[derive(Clone, Debug)]
pub enum LibraryMessage {
    SearchChanged(String),
    ViewModeChanged(GameViewMode),
    LaunchRequested(String),
    LaunchFinished { game_id: String, success: bool },
    EditRequested(String),
    DeleteRequested(String),
}

#[derive(Clone, Debug, Default)]
pub struct LibraryState {
    games: Vec<GameSummary>,
    pub search_query: String,
    pub view_mode: GameViewMode,
    launching: HashSet<String>,
}

impl LibraryState {
    pub fn with_games(games: Vec<GameSummary>) -> Self {
        Self {
            games,
            ..Self::default()
        }
    }

    pub fn games(&self) -> &[GameSummary] {
        &self.games
    }

    pub fn replace_games(&mut self, games: Vec<GameSummary>) {
        self.games = games;
    }

    pub fn apply_game(&mut self, game: GameSummary) {
        if let Some(existing) = self
            .games
            .iter_mut()
            .find(|existing| existing.id == game.id)
        {
            *existing = game;
        } else {
            self.games.push(game);
        }
    }

    pub fn remove_game(&mut self, game_id: &str) {
        self.games.retain(|game| game.id != game_id);
        self.launching.remove(game_id);
    }

    pub fn filtered_games(&self) -> Vec<&GameSummary> {
        let query = self.search_query.trim().to_lowercase();
        self.games
            .iter()
            .filter(|game| {
                query.is_empty()
                    || game.title.to_lowercase().contains(&query)
                    || game.engine_type.to_lowercase().contains(&query)
                    || game.game_type.to_lowercase().contains(&query)
            })
            .collect()
    }

    pub fn start_launch(&mut self, game_id: &str) -> bool {
        self.games.iter().any(|game| game.id == game_id)
            && self.launching.insert(game_id.to_owned())
    }

    pub fn finish_launch(&mut self, game_id: &str) {
        self.launching.remove(game_id);
    }

    pub fn is_launching(&self, game_id: &str) -> bool {
        self.launching.contains(game_id)
    }

    pub fn apply(&mut self, message: LibraryMessage) {
        match message {
            LibraryMessage::SearchChanged(query) => self.search_query = query,
            LibraryMessage::ViewModeChanged(mode) => self.view_mode = mode,
            LibraryMessage::LaunchRequested(game_id) => {
                self.start_launch(&game_id);
            }
            LibraryMessage::LaunchFinished {
                game_id,
                success: _,
            } => self.finish_launch(&game_id),
            LibraryMessage::EditRequested(_) | LibraryMessage::DeleteRequested(_) => {}
        }
    }
}
