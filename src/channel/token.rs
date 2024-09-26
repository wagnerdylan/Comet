pub(super) trait ChannelTokenOps {
    fn new(accessor_id: usize) -> Self;

    fn get_accessor_id(&self) -> usize;

    fn is_valid(&self) -> bool;
}

#[derive(Default)]
pub struct ChannelOwnerToken {
    accessor_id: usize,
    is_valid: bool,
}

#[derive(Default)]
pub struct ChannelReaderToken {
    accessor_id: usize,
    is_valid: bool,
}

pub struct ChannelBehindToken {
    accessor_id: usize,
    is_valid: bool,
}

impl ChannelTokenOps for ChannelOwnerToken {
    fn new(accessor_id: usize) -> Self {
        Self {
            accessor_id,
            is_valid: true,
        }
    }

    fn get_accessor_id(&self) -> usize {
        self.accessor_id
    }

    fn is_valid(&self) -> bool {
        self.is_valid
    }
}

impl ChannelTokenOps for ChannelReaderToken {
    fn new(accessor_id: usize) -> Self {
        Self {
            accessor_id,
            is_valid: true,
        }
    }

    fn get_accessor_id(&self) -> usize {
        self.accessor_id
    }

    fn is_valid(&self) -> bool {
        self.is_valid
    }
}

impl ChannelTokenOps for ChannelBehindToken {
    fn new(accessor_id: usize) -> Self {
        Self {
            accessor_id,
            is_valid: true,
        }
    }

    fn get_accessor_id(&self) -> usize {
        self.accessor_id
    }

    fn is_valid(&self) -> bool {
        self.is_valid
    }
}
