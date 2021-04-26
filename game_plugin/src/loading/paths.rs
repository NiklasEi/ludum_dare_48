pub struct AssetPaths {
    pub fira_sans: &'static str,
    pub audio_fall: &'static str,
    pub audio_button_click: &'static str,
    pub audio_wall_moving: &'static str,
    pub audio_ground_background: &'static str,
    pub audio_dirt_background: &'static str,
    pub audio_stone_background: &'static str,
    pub audio_lava_background: &'static str,
    pub texture_player: &'static str,
    pub texture_acorn: &'static str,
    pub texture_button: &'static str,
    pub texture_button_active: &'static str,
}

pub const PATHS: AssetPaths = AssetPaths {
    fira_sans: "fonts/FiraSans-Bold.ttf",
    audio_fall: "audio/fall.ogg",
    audio_button_click: "audio/button_click.ogg",
    audio_wall_moving: "audio/wall_moving.ogg",
    audio_ground_background: "audio/happy_background.ogg",
    audio_dirt_background: "audio/dirt_background.ogg",
    audio_stone_background: "audio/stone_background.ogg",
    audio_lava_background: "audio/lava_background.ogg",
    texture_player: "textures/player.png",
    texture_acorn: "textures/acorn.png",
    texture_button: "textures/button.png",
    texture_button_active: "textures/button_active.png",
};
