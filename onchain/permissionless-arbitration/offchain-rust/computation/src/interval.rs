fn istrides() {

}

fn iustrides() {

}

struct StrideCounter {
    interval: i32,
    total: i32,
    value: i32, 
}

impl StrideCounter {
    fn new(interval: i32, total: i32, v: Option<i32>) -> Self {
        let v = v.unwrap_or(-1);
        StrideCounter {
            interval,
            total,
            value: v,
        }
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    /*fn cycle(&self) -> i32 {
        let cycle = self.value << (self.interval.log2_stride() - constants::A);
        cycle
    }

    fn ucycle(&self) -> i32 {
        let ucycle = self.value << self.interval.log2_stride();
        ucycle
    }*/

    fn remaining_strides(&self) -> i32 {
        self.total - self.value
    }
}