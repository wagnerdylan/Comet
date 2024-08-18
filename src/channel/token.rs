pub trait ChannelTokenOps {
    fn new(accessor_id: usize) -> Self;

    fn get_accessor_id(&self) -> usize;
}

pub struct ChannelOwnerToken {
    accessor_id: usize,
}

pub struct ChannelReaderToken {
    accessor_id: usize,
}

impl ChannelTokenOps for ChannelOwnerToken {
    fn new(accessor_id: usize) -> Self {
        Self { accessor_id }
    }

    fn get_accessor_id(&self) -> usize {
        self.accessor_id
    }
}
impl ChannelTokenOps for ChannelReaderToken {
    fn new(accessor_id: usize) -> Self {
        Self { accessor_id }
    }

    fn get_accessor_id(&self) -> usize {
        self.accessor_id
    }
}
