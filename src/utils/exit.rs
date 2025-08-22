use colored::*;

use crate::utils::random::get_random_u128;

pub fn get_exit_message(config: &crate::Config) -> String
{
        if !config.show_exit_message
        {
                return String::from("");
        }

        let messages = [
                ("Oxide has been reduced to its elemental components.", "red"),
                ("Oxide underwent catastrophic reduction and ceased to exist.", "magenta"),
                ("Oxide has been violently reduced back to base metal.", "yellow"),
                ("Oxide experienced spontaneous deoxygenation and met its end.", "cyan"),
                ("Oxide's oxidation state was permanently set to zero.", "green"),
                ("Oxide was stripped of its oxygen atoms and left for dead.", "blue"),
                ("Oxide underwent irreversible reduction at room temperature.", "purple"),
                ("Oxide's reaction has reached equilibrium... with the void.", "white"),
                ("Oxide's half-life has expired.", "bright red"),
                ("Oxide has been oxidized out of existence.", "bright yellow"),
        ];

        let choice: u128 = get_random_u128(messages.len() as u128).unwrap();

        let (message, color) = messages.get(choice as usize).unwrap();

        match *color
        {
                "red" => message.red().to_string(),
                "magenta" => message.magenta().to_string(),
                "yellow" => message.yellow().to_string(),
                "cyan" => message.cyan().to_string(),
                "green" => message.green().to_string(),
                "blue" => message.blue().to_string(),
                "purple" => message.purple().to_string(),
                "white" => message.white().to_string(),
                "bright red" => message.bright_red().to_string(),
                "bright yellow" => message.bright_yellow().to_string(),
                _ => message.to_string(),
        }
}
