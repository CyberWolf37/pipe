pub mod default {
    use crate::PipeState;
    use crate::PipeU;

    pub fn function_control<'r,T: PipeU>(pipe_u: &'r T) -> PipeState {
        PipeState::ConsumeState
    }
    
    pub fn function_core<'r,T: PipeU>(pipe_u: &'r T) -> PipeState {
        println!("function core consume");
        PipeState::ConsumeState
    }
}

