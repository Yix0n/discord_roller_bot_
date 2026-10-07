use serenity::all::Guild;

const SERVER_WHITELIST: [u64; 4] = [
    1528879518201151678, // 4m
    1383937008660906106, // 3200 Skill Cannon
    1467642118850744404, // Aik Server
    1461870566519738562, // Test server
];

pub fn bot_activation() {

}

pub fn is_server_whitelisted(guild: &Guild) -> bool {

    let server_name = &guild.name;
    println!("SERVER: {}", format!("{} - {}", server_name, guild.id));

    SERVER_WHITELIST.contains(&guild.id.get())
}