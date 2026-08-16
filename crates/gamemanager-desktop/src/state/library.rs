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
        let mut state = Self {
            games,
            ..Self::default()
        };
        state.sort_games();
        state
    }

    pub fn games(&self) -> &[GameSummary] {
        &self.games
    }

    pub fn replace_games(&mut self, games: Vec<GameSummary>) {
        self.games = games;
        self.sort_games();
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
        self.sort_games();
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

    pub fn relative_played_time(last_played_at: Option<i64>, now: i64) -> Option<String> {
        let timestamp = last_played_at?;
        let seconds = now.saturating_sub(timestamp);
        let minutes = seconds / 60;
        let hours = minutes / 60;
        let days = hours / 24;

        Some(if days > 0 {
            format!("{days} 天前")
        } else if hours > 0 {
            format!("{hours} 小时前")
        } else if minutes > 0 {
            format!("{minutes} 分钟前")
        } else {
            "刚刚".to_owned()
        })
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

    fn sort_games(&mut self) {
        self.games
            .sort_by_key(|game| std::cmp::Reverse(game.last_played_at.unwrap_or(game.created_at)));
    }
}
