pub enum ServerCommand {
    Hello { command_type: u8, udp_port: u16 },
    SetStation {
        command_type: u8,
        station_number: u16,
    },
}

// Server to Client Replies

pub struct Welcome {
    pub reply_type: u8,
    pub num_stations: u16,
}

pub struct Announce {
    pub reply_type: u8,
    pub song_name_size: u8,
    pub song_name: [u8],
}

pub struct InvalidCommand {
    pub reply_type: u8,
    pub reply_string_size: u8,
    pub reply_string: [u8],
}
