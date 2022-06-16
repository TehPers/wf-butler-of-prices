mod application_commands;
mod interaction_handler;
mod slash_command;
mod snowflake_resolver;

pub use application_commands::*;
pub use interaction_handler::*;
pub use slash_command::*;
pub use snowflake_resolver::*;

pub fn interactions_module() -> runtime_injector::Module {
    use self::{ApplicationCommandHandler, InteractionHandler};
    use runtime_injector::{IntoFallible, IntoSingleton};

    runtime_injector::define_module! {
        services = [
            ApplicationCommandHandler::new.fallible().singleton(),
        ]
    }
}
