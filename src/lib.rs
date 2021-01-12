mod model;

use std::sync::Arc;


#[derive(Clone)]
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

trait PipeB<T: PipeU> {
    fn function_control(&self) -> Option<Arc<dyn Fn(&T) -> PipeState + Send + Sync>>;
    fn function_core(&self) -> Option<Arc<dyn Fn(&T) -> PipeState + Send + Sync>>;
    fn internal_state(&self) -> PipeState;

    fn control(&self,pipe_u: &T) -> PipeState {
        match self.function_control().clone() {
            Some(e) => (e.clone())(pipe_u),
            None => PipeState::NextState,
        }
    }

    fn consume(&self,pipe_u: &T) -> PipeState {

        match self.function_core().clone() {
            Some(e) => (e.clone())(pipe_u),
            None => panic!("Don't have function to consume"),
        }
    }
}

struct Pipe<'r, T: PipeU, U: PipeB<T>> {
    pipeArrayBox: Vec<U>,
    pipeArrayUsr: Vec<(&'r T,usize)>,
}

impl<'r,T,U> Pipe<'r,T,U> where T : PipeU, U : PipeB<T>
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

    fn push_box(mut self,pipe_box: U) -> Self {
        self.pipeArrayBox.push(pipe_box);
        self
    }

    fn get_box(&self, pipe_u: &T) -> &U {
        let index_box: Option<&U> = match self.pipeArrayUsr.iter().find(|x| x.0 == pipe_u) {
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
    use crate::model::models::PipeBox;
    use crate::model::models::PipeUser;
    use crate::PipeState;

    #[test]
    fn it_works() {

        let user_01 = PipeUser::new("Hello, World !");
        let user_02 = PipeUser::new("Il fait chaud ici !");

        let box_1 = PipeBox::new();
        let box_2 = PipeBox::new();

        
        let mut pipe = Pipe::new()
                            .push_box(box_1)
                            .push_box(box_2);

        pipe.push_user(&user_01);
        pipe.push_user(&user_02);
    }
}
