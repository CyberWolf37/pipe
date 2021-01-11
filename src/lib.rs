use std::sync::Arc;


enum PipeState {
    NextState,
    RestartState,
    WaitState,
    ConsumeState,
}

trait PipeU: Eq {
    type Item;

    fn get_item(&self) -> &Self::Item;
}

struct PipeBox<T: PipeU> {
    function_control: Option<Arc<dyn Fn(&T) -> PipeState + Send + Sync>>,
    function_core: Option<Arc<dyn Fn(&T) -> PipeState + Send + Sync>>,
    internal_state: PipeState,
}

impl<T: PipeU> PipeBox<T> {
    fn new() -> Self {
        PipeBox {
            function_control: None,
            function_core: None,
            internal_state: PipeState::ConsumeState,
        }
    }

    fn internal_state(&self) -> &PipeState {
        &self.internal_state
    }

    fn consume(&self,pipe_u: &T) -> PipeState {

        match self.function_core.clone() {
            Some(e) => (e.clone())(pipe_u),
            None => panic!("Don't have function to consume"),
        }
    }

    fn control(&self, pipe_u: &T) -> PipeState {
        match self.function_control.clone() {
            Some(e) => (e.clone())(pipe_u),
            None => PipeState::NextState,
        }
    }

    fn set_control<U: Fn(&T) -> PipeState + Send + Sync + 'static>(&mut self,func: U) -> &Self {
        self.function_control = Some(Arc::new(func));
        self
    }

    fn set_consume<U: Fn(&T) -> PipeState + Send + Sync + 'static>(&mut self,func: U) -> &Self {
        self.function_core = Some(Arc::new(func));
        self
    }

    fn set_internal_state(&mut self, state: PipeState) {
        self.internal_state = state;
    }
}

struct Pipe<'r, T: PipeU> {
    pipeArrayBox: Vec<PipeBox<T>>,
    pipeArrayUsr: Vec<(&'r T,usize)>,
}

impl<'r,T> Pipe<'r,T> where T : PipeU
{
    fn new() -> Self {
        Pipe {
            pipeArrayBox: Vec::new(),
            pipeArrayUsr: Vec::new(),
        }
    }

    fn push_user(&mut self,pipe_t: &'r T) {
        if self.pipeArrayUsr.iter().find(|x| x.0 == pipe_t).is_none() {
            self.pipeArrayUsr.push((pipe_t,0));
            self.consume(pipe_t);
        }
        else {
            self.consume(pipe_t);
        }
        
    }

    fn remove_user(&mut self, pipe_t: &'r T) {
        let index:Option<usize> = match self.pipeArrayUsr.iter_mut().enumerate().find(|x| x.1.0 == pipe_t) {
            Some(e) => {
                Some(e.0)
            },
            None => None
        };

        match index {
            Some(e) => {
                self.pipeArrayUsr.remove(e);
            },
            None => {}
        }
    }

    fn push_box(mut self,pipe_box: PipeBox<T>) -> Self {
        self.pipeArrayBox.push(pipe_box);
        self
    }

    fn get_box(&self, pipe_u: &T) -> &PipeBox<T> {
        let index_box: Option<&PipeBox<T>> = match self.pipeArrayUsr.iter().find(|x| x.0 == pipe_u) {
            Some(e) => Some(&self.pipeArrayBox[e.1]),
            None => panic!("Don't have this box for user"),
        };

        index_box.unwrap()
    }

    fn set_user(&mut self,pipe_u: &T,index_box: usize) {
        match self.pipeArrayUsr.iter_mut().find(|x| x.0 == pipe_u) {
            Some(e) => Some(e.1 = index_box),
            None => panic!("Don't have this user"),
        };
    }

    fn consume(&mut self,pipe_u: &'r T) {
        let index_box: Option<usize> = match self.pipeArrayUsr.iter().find(|x| x.0 == pipe_u) {
            Some(e) => Some(e.1),
            None => panic!("Don't have this box for user"),
        };

        match self.get_box(pipe_u).control(pipe_u) {
            PipeState::ConsumeState => {
                match self.get_box(pipe_u).consume(pipe_u) {
                    // Si c'est bon on passe au box suivant
                    PipeState::NextState => {
                        let index_box = index_box.unwrap() + 1;
                        let index_arr = self.pipeArrayBox.len();

                        self.set_user(pipe_u, index_box);

                        if index_box >= index_arr {
                            self.remove_user(pipe_u);
                        }
                        else {
                            match self.get_box(pipe_u).internal_state() {
                                PipeState::ConsumeState => self.consume(pipe_u),
                                PipeState::WaitState => {},
                                _ => {}
                            }
                        }
                        
                    }
                    PipeState::RestartState => {}
                    _ => {}
                }
            }

            PipeState::RestartState => {}
            _ => {}
        }
    }

}

#[cfg(test)]
mod tests {

    use crate::Pipe;
    use crate::PipeBox;
    use crate::PipeU;
    use crate::PipeState;

    #[test]
    fn it_works() {

        struct User {
            say: String
        }

        impl PartialEq for User {
            fn eq(&self, other: &Self) -> bool {
                self.say == other.say
            }
        }

        impl Eq for User {}

        impl PipeU for User {
            type Item = User;

            fn get_item(&self) -> &Self::Item{
                self
            }
        }

        let mut box_1 = PipeBox::<User>::new();
        box_1.set_control(|x| {println!("1 :{}",x.say);PipeState::ConsumeState});
        box_1.set_consume(|x| {println!("1 :{}",x.say);PipeState::NextState});

        let mut box_2 = PipeBox::<User>::new();
        box_2.set_control(|x| {println!("2 :{}",x.say);PipeState::ConsumeState});
        box_2.set_consume(|x| {println!("2 :{}",x.say);PipeState::NextState});

        let mut box_3 = PipeBox::<User>::new();
        box_3.set_control(|x| {
            match x.say.as_str() {
                "Hello mother fucker" => {println!("3 : Quelle grossier personnage");PipeState::ConsumeState},
                "Fucking ass hole" => {println!("3: Trou du cul toi même");PipeState::ConsumeState},
                _ => {println!("Mouais pas compris ?");PipeState::ConsumeState},
            }
        });

        box_3.set_consume(|x| {
            match x.say.as_str() {
                "Hello mother fucker" => {println!("3 : Ton cul c'est du poulet");PipeState::ConsumeState},
                "Fucking ass hole" => {println!("3: Ok gogole");PipeState::ConsumeState},
                _ => {println!("Mouais pas compris ?");PipeState::ConsumeState},
            }
        });
        
        let mut pipe = Pipe::new()
                            .push_box(box_1)
                            .push_box(box_2)
                            .push_box(box_3);

        let u_1 = User{say:String::from("Hello mother fucker")};
        let u_2 = User{say:String::from("Fucking ass hole")};

        pipe.push_user(&u_1);
        pipe.push_user(&u_2);
    }
}
