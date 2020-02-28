#![feature(ip)]
#![feature(str_strip)]

use clap;

mod network;
mod steam;


fn main() -> Result<(), &'static str> {
    let yaml = clap::load_yaml!("cli.yml");
    let app = clap::App::from_yaml(yaml).setting(clap::AppSettings::ArgRequiredElseHelp);
    let cmd_main = &app.get_matches();

    if cmd_main.subcommand_matches("ping").is_some() {
        println!("Pong!");
        return Ok(());
    }

    else if cmd_main.subcommand_matches("network").is_some() {
        network::network_command()
    }

    else if let Some(cmd_steam) = cmd_main.subcommand_matches("steam") {

        if let Some(cmd_list) = cmd_steam.subcommand_matches("list") {
            steam::steam_list_command(&cmd_list.value_of("steamapps"))
        }

        else if let Some(cmd_backup) = cmd_steam.subcommand_matches("backup") {
            steam::steam_backup_command(&cmd_backup.value_of("steamapps"), &cmd_backup.value_of("destination"), &cmd_backup.values_of("appids"))
        }

        else if let Some(cmd_restore) = cmd_steam.subcommand_matches("restore") {
            steam::steam_restore_command(&cmd_restore.value_of("steamapps"), &cmd_restore.value_of("source"), &cmd_restore.values_of("appids"))
        }

        else {
            Err("Unknown subcommand")
        }
    }

    else {
        Err("Unknown subcommand")
    }
}
