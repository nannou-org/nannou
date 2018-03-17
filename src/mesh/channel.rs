use std::borrow::{Borrow, Cow};

/// Types that may be used as a data channel within a mesh.
pub trait Channel {
    /// The type contained within the channel.
    type Element;
    /// Borrow the data channel.
    fn channel(&self) -> &[Self::Element];
}

/// Types that may be used as a data channel within a mesh.
pub trait ChannelMut: Channel {
    /// Mutably borrow the data channel.
    fn channel_mut(&mut self) -> &mut [Self::Element];
}

// /// Types that may be used as a constant-length buffer underlying a `Bounded` ring buffer.
// pub trait FixedSizeArray {
//     /// The constant length.
//     const LEN: usize;
// }

impl<'a, T> Channel for &'a [T] {
    type Element = T;
    #[inline]
    fn channel(&self) -> &[Self::Element] {
        self
    }
}

impl<'a, T> Channel for &'a mut [T] {
    type Element = T;
    #[inline]
    fn channel(&self) -> &[Self::Element] {
        self
    }
}

impl<'a, T> ChannelMut for &'a mut [T] {
    #[inline]
    fn channel_mut(&mut self) -> &mut [Self::Element] {
        self
    }
}

impl<T> Channel for Box<[T]> {
    type Element = T;
    #[inline]
    fn channel(&self) -> &[Self::Element] {
        &self[..]
    }
}

impl<T> ChannelMut for Box<[T]> {
    #[inline]
    fn channel_mut(&mut self) -> &mut [Self::Element] {
        &mut self[..]
    }
}

impl<T> Channel for Vec<T> {
    type Element = T;
    #[inline]
    fn channel(&self) -> &[Self::Element] {
        &self[..]
    }
}

impl<T> ChannelMut for Vec<T> {
    #[inline]
    fn channel_mut(&mut self) -> &mut [Self::Element] {
        &mut self[..]
    }
}

impl<'a, T> Channel for Cow<'a, [T]>
where
    [T]: ToOwned,
{
    type Element = T;
    #[inline]
    fn channel(&self) -> &[Self::Element] {
        self.borrow()
    }
}

macro_rules! impl_channel_for_arrays {
    ($($N:expr)*) => {
        $(
            impl<T> Channel for [T; $N] {
                type Element = T;
                #[inline]
                fn channel(&self) -> &[Self::Element] {
                    &self[..]
                }
            }
            impl<T> ChannelMut for [T; $N] {
                #[inline]
                fn channel_mut(&mut self) -> &mut [Self::Element] {
                    &mut self[..]
                }
            }
            // impl<T> FixedSizeArray for [T; $N] {
            //     const LEN: usize = $N;
            // }
        )*
    };
}

impl_channel_for_arrays! {
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34
    35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63 64 65
    66 67 68 69 70 71 72 73 74 75 76 77 78 79 80 81 82 83 84 85 86 87 88 89 90 91 92 93 94 95 96
    97 98 99 100 101 102 103 104 105 106 107 108 109 110 111 112 113 114 115 116 117 118 119 120
    121 122 123 124 125 126 127 128 256 512 1024 2048 4096 8192
}
