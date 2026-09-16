#[cfg(not(target_arch = "wasm32"))]
use tokio::task::{ JoinHandle, JoinError };
use std::future::Future;
#[cfg(target_arch = "wasm32")]
use std::{ rc::Rc, cell::RefCell };
use std::fmt;

#[derive(Debug)]
pub enum ClientTaskError {
    #[cfg(not(target_arch = "wasm32"))]
    Join(JoinError),

    NotReady
}

impl fmt::Display for ClientTaskError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            ClientTaskError::NotReady => {
                write!(f, "Task is not ready yet")
            }

            #[cfg(not(target_arch = "wasm32"))]
            ClientTaskError::Join(error) => {
                write!(f, "Task failed to join: {}", error)
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct ClientTask<T> {
    handle: JoinHandle<T>
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + 'static> ClientTask<T> {
    pub fn spawn(
        runtime: &tokio::runtime::Runtime,
        future: impl std::future::Future<Output = T> + Send + 'static
    ) -> Self {
        Self {
            handle: runtime.spawn(future)
        }
    }

    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
    pub fn take(self, runtime: &tokio::runtime::Runtime) -> Result<T, ClientTaskError> {
        runtime.block_on(self.handle).map_err(ClientTaskError::Join)
    }
}


#[cfg(target_arch = "wasm32")]
pub struct ClientTask<T> {
    result: Rc<RefCell<Option<T>>>,
}
#[cfg(target_arch = "wasm32")]
impl<T: 'static> ClientTask<T> {
    pub fn spawn(future: impl Future<Output = T> + 'static) -> Self {
        let result: Rc<RefCell<Option<T>>> = Rc::new(std::cell::RefCell::new(None));

        let task_result: Rc<RefCell<Option<T>>> = result.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let output: T = future.await;
            *task_result.borrow_mut() = Some(output);
        });
        Self { result }
    }

    pub fn is_finished(&self) -> bool {
        self.result.borrow().is_some()
    }
    pub fn take(self) -> Result<T, ClientTaskError> {
        match self.result.borrow_mut().take() {
            Some(result) => Ok(result),
            None         => Err(ClientTaskError::NotReady),
        }
    }
}
