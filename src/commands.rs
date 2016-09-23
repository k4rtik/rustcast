// Client to Server Commands
pub enum ServerCommand {
    Hello { command_type: u8, udp_port: u16 },
    SetStation {
        command_type: u8,
        station_number: u16,
    },
    Invalid { command_type: u8, unused: u16 },
}

// Server to Client Replies
#[allow(dead_code)]
pub struct Welcome {
    pub reply_type: u8,
    pub num_stations: u16,
}

#[allow(dead_code)]
pub struct Announce {
    pub reply_type: u8,
    pub song_name_size: u8,
    pub song_name: [u8],
}

#[allow(dead_code)]
pub struct InvalidCommand {
    pub reply_type: u8,
    pub reply_string_size: u8,
    pub reply_string: [u8],
}
