use bevy::prelude::*;
use bombombo::AppPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}
