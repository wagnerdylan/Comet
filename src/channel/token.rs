pub struct ChannelOwnerToken {
    owner_token_id: usize,
    accessor_id: usize,
}

impl ChannelOwnerToken {
    pub fn new(owner_token_id: usize, accessor_id: usize) -> Self {
        Self {
            owner_token_id,
            accessor_id,
        }
    }

    pub fn get_owner_token_id(&self) -> usize {
        self.owner_token_id
    }

    pub fn get_accessor_id(&self) -> usize {
        self.accessor_id
    }
}

pub struct ChannelReaderToken {
    accessor_id: usize,
}

impl ChannelReaderToken {
    pub fn new(accessor_id: usize) -> Self {
        Self { accessor_id }
    }

    pub fn get_accessor_id(&self) -> usize {
        self.accessor_id
    }
}
