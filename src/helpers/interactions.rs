use serenity::all::{ChannelId, CommandDataOption, CommandDataOptionValue, UserId};

pub fn get_string_option(options: &Vec<CommandDataOption>, name: &str) -> Option<String> {
    options.iter()
        .find(|option| option.name == name)
        .and_then(|opt| opt.value.as_str())
        .map(String::from)
}

pub fn get_integer_option(options: &Vec<CommandDataOption>, name: &str) -> Option<i64> {
    let r = options.iter()
        .find(|option| option.name == name)
        .map(|opt| opt.value.as_i64());

    r?
}

pub fn get_float_option(options: &Vec<CommandDataOption>, name: &str) -> Option<f64> {
    let r = options.iter()
    .find(|option| option.name == name)
    .map(|opt| opt.value.as_f64());
    
    r?
}

pub fn get_boolean_option(options: &Vec<serenity::all::CommandDataOption>, name: &str) -> Option<bool> {
    options.iter()
        .find(|option| option.name == name)
        .and_then(|opt| opt.value.as_bool())
}

pub fn get_user_option(options: &Vec<CommandDataOption>, name: &str) -> Option<UserId> {
    options.iter()
        .find(|option| option.name == name)
        .and_then(|opt| opt.value.as_user_id())
}

pub fn get_channel_option(options: &Vec<CommandDataOption>, name: &str) -> Option<ChannelId> {
    options.iter()
    .find(|option| option.name == name)
        .and_then(|opt| opt.value.as_channel_id())
}

pub fn get_subcommand_option(options: &Vec<CommandDataOption>) -> (String, Vec<CommandDataOption>) {
    let cmd = options.get(0).unwrap().clone();
    match cmd.value {
        CommandDataOptionValue::SubCommand(data) |
        CommandDataOptionValue::SubCommandGroup(data) => (cmd.name, data),

        _ => panic!("This should be invoked ONLY when subcommands are used")
    }
}