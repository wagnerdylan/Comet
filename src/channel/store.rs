use super::{reg::Reg, token::ChannelToken};

struct Channel<'a> {
    pub name: &'a str,
    pub owner_token: usize,
    pub reg: Reg,
}

pub struct ChannelStore<'a, const N: usize> {
    channels: [Option<Channel<'a>>; N],
    current_size: usize,
    init_complete: bool,
}

impl<'a, const N: usize> ChannelStore<'a, N> {
    fn get_existing_channel_accessor_id(&self, name: &'a str) -> Result<usize, ()> {
        for (i, channel_o) in self.channels.iter().enumerate() {
            if let Some(some_channel) = channel_o {
                if some_channel.name == name {
                    return Ok(i);
                }
            }
        }

        Err(())
    }

    fn is_unique_channel_name(&self, name: &'a str) -> bool {
        let query_result = self.get_existing_channel_accessor_id(name);

        query_result.is_err()
    }

    fn register_channel(&mut self, name: &'a str, reg: Reg) -> (usize, usize) {
        assert!(self.current_size < self.channels.len());

        let owner_token = self.current_size;
        let accessor_id = self.current_size;
        self.channels[self.current_size] = Some(Channel {
            name,
            owner_token,
            reg,
        });
        self.current_size += 1;

        (owner_token, accessor_id)
    }

    pub fn register_write_channel(&mut self, name: &'a str, reg: Reg) -> ChannelToken {
        assert!(!self.init_complete);
        assert!(self.is_unique_channel_name(name));
        let (owner_token, accessor_id) = self.register_channel(name, reg);
        ChannelToken::new(Some(owner_token), accessor_id)
    }

    pub fn register_read_channel(&self, name: &'a str) -> ChannelToken {
        assert!(!self.init_complete);
        let query_result = self.get_existing_channel_accessor_id(name);
        // TODO handle panic in a better way here.
        let accessor_id = query_result.unwrap();
        ChannelToken::new(None, accessor_id)
    }
}

impl<'a, const N: usize> Default for ChannelStore<'a, N> {
    fn default() -> Self {
        Self {
            channels: [const { None }; N],
            current_size: 0,
            init_complete: false,
        }
    }
}
