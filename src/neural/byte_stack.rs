pub struct ByteStack {
    data: [u8; 256],
    sp: u8,
}

impl ByteStack {
    pub fn new() -> Self {
        Self {
            data: [0; 256],
            sp: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, val: u8) {
        self.data[self.sp as usize] = val;
        self.sp = self.sp.wrapping_add(1);
    }

    #[inline(always)]
    pub fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_sub(1);
        let val = self.data[self.sp as usize];
        self.data[self.sp as usize] = 0;
        val
    }

    #[inline(always)]
    pub fn peek(&self) -> u8 {
        self.data[self.sp.wrapping_sub(1) as usize]
    }

    #[inline(always)]
    pub fn swap(&mut self) {
        let a = self.sp.wrapping_sub(1) as usize;
        let b = self.sp.wrapping_sub(2) as usize;
        self.data.swap(a, b);
    }

    #[inline(always)]
    pub fn over(&mut self) {
        let val = self.data[self.sp.wrapping_sub(2) as usize];
        self.push(val);
    }
}
