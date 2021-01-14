// Models PipeBox and PipeUser
pub mod models {

    use crate::PipeB;
    use crate::PipeU;
    use crate::PipeState;
    use std::sync::Arc;


    pub struct PipeBox {
        function_control: Arc<dyn Fn(&PipeUser) -> PipeState + Send + Sync>,
        function_core: Arc<dyn Fn(&PipeUser) -> PipeState + Send + Sync>,
        internal_state: PipeState,
    }

    impl PipeB<PipeUser> for PipeBox {
        fn function_control(&self) -> Option<Arc<dyn Fn(&PipeUser) -> PipeState + Send + Sync>> {
            let func = self.function_control.clone();
            Some(func)
        }
        fn function_core(&self) -> Option<Arc<dyn Fn(&PipeUser) -> PipeState + Send + Sync>> {
            let func = self.function_core.clone();
            Some(func)
        }
        fn internal_state(&self) -> PipeState {
            self.internal_state.clone()
        }
    }

    impl PipeBox {
        pub fn new() -> Self {
            PipeBox {
                function_control: Arc::new(|x: &PipeUser| {println!("PipeBox Controle :{}",x.get_item());PipeState::ConsumeState}),
                function_core: Arc::new(|x: &PipeUser| {println!("PipeBox Core :{}",x.get_item());PipeState::NextState}),
                internal_state: PipeState::ConsumeState,
            }
        }

        fn internal_state(&self) -> &PipeState {
            &self.internal_state
        }

        pub fn set_control<U: Fn(&PipeUser) -> PipeState + Send + Sync + 'static>(mut self,func: U) -> Self {
            self.function_control = Arc::new(func);
            self
        }

        pub fn set_consume<U: Fn(&PipeUser) -> PipeState + Send + Sync + 'static>(mut self,func: U) -> Self {
            self.function_core = Arc::new(func);
            self
        }

        pub fn set_internal_state(&mut self, state: PipeState) {
            self.internal_state = state;
        }
    }

    pub struct PipeUser {
        say: String
    }

    impl PartialEq for PipeUser {
        fn eq(&self, other: &Self) -> bool {
            self.say == other.say
        }
    }

    impl PipeU for PipeUser {
        type Item = String;

        fn get_item(&self) -> &Self::Item{
            &self.say
        }
    }

    impl PipeUser {
        pub fn new(say: &str) -> Self {
            PipeUser{
                say: say.to_string() 
            }
        }
    }
}
