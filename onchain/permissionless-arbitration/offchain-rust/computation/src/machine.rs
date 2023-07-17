struct Machine {
    machine: cartesi::Machine,
    cycle: u64,
    ucycle: u64,
    base_cycle: u64,
}

impl Machine {
    fn new_from_path(path: &str) -> Result<Machine, String> {
        let machine = cartesi::machine(path)?;
        let start_cycle = machine.read_mcycle();

        assert!(machine.read_uarch_cycle() == 0);

        let b = Machine {
            machine,
            cycle: 0,
            ucycle: 0,
            base_cycle: start_cycle,
        };

        Ok(b)
    }

    fn result() {}
    fn advance(&self, cycle: u64) {}
    fn uadvance(&self, cycle: u64) {}
    fn ureset(&self) {}

}