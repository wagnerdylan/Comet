pub struct ChannelToken {
    owner_token: Option<usize>,
    accessor_id: usize,
}

impl ChannelToken {
    pub fn new(owner_token: Option<usize>, accessor_id: usize) -> Self {
        Self {
            owner_token,
            accessor_id,
        }
    }
}
