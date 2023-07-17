struct ComputationResult {
    state: State,
    halted: bool,
    uhalted: bool,
}

impl ComputationResult {
    fn new(state: State, halted: bool, uhalted: bool) -> ComputationResult {
        ComputationResult {
            state,
            halted,
            uhalted,
        }
    }
}