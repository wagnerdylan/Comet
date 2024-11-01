use core::marker::PhantomData;

pub(super) trait ChannelTokenOps {
    fn new(accessor_id: usize) -> Self;

    fn get_accessor_id(&self) -> usize;

    fn is_valid(&self) -> bool;
}

/// Channel owner token used to access a channel with read and write permissions.
#[derive(Default)]
pub struct ChannelOwnerToken<T> {
    accessor_id: usize,
    is_valid: bool,
    phantom_marker: PhantomData<T>,
}

/// Channel owner token used to access a channel with read only permissions.
#[derive(Default)]
pub struct ChannelReaderToken<T> {
    accessor_id: usize,
    is_valid: bool,
    phantom_marker: PhantomData<T>,
}

/// Channel behind token used to read the previous channel value.
#[derive(Default)]
pub struct ChannelBehindToken<T> {
    accessor_id: usize,
    is_valid: bool,
    phantom_marker: PhantomData<T>,
}

impl<T> ChannelTokenOps for ChannelOwnerToken<T> {
    fn new(accessor_id: usize) -> Self {
        Self {
            accessor_id,
            is_valid: true,
            phantom_marker: PhantomData,
        }
    }

    fn get_accessor_id(&self) -> usize {
        self.accessor_id
    }

    fn is_valid(&self) -> bool {
        self.is_valid
    }
}

impl<T> ChannelTokenOps for ChannelReaderToken<T> {
    fn new(accessor_id: usize) -> Self {
        Self {
            accessor_id,
            is_valid: true,
            phantom_marker: PhantomData,
        }
    }

    fn get_accessor_id(&self) -> usize {
        self.accessor_id
    }

    fn is_valid(&self) -> bool {
        self.is_valid
    }
}

impl<T> ChannelTokenOps for ChannelBehindToken<T> {
    fn new(accessor_id: usize) -> Self {
        Self {
            accessor_id,
            is_valid: true,
            phantom_marker: PhantomData,
        }
    }

    fn get_accessor_id(&self) -> usize {
        self.accessor_id
    }

    fn is_valid(&self) -> bool {
        self.is_valid
    }
}
