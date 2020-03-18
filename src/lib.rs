use {
    async_std::{
        prelude::*,
        net::{
            TcpListener,
        },
        sync::{
            Arc,
        },
        task::{
            spawn,
        },
    },
    async_h1::{
        accept,
    },
};

pub use {
    http_types::{Request, Response, Error, StatusCode}
};

pub struct Server<F> {
    socket: String, 
    handler: Arc<F>,
}

impl<F, Fut> Server<F>
where
    F: Fn(Request) -> Fut,
    F: Send + Sync + 'static,
    Fut: Future<Output = Result<Response, Error>>,
    Fut: Send,
{
    pub fn new(addr: &str, handler: F) -> Self {
        Server {
            socket: addr.to_string(), 
            handler: Arc::new(handler),
        }
    }
    pub async fn run(self) -> Result<(), Error> {
        let listener = TcpListener::bind(self.socket).await?;
        let addr = format!("http://{}", listener.local_addr()?); 
        let mut incoming = listener.incoming();
        
        while let Some(stream) = incoming.next().await {
            let handler = self.handler.clone();
            let addr = addr.clone();
            let stream = stream?;
            
            spawn(async move {
                if let Err(err) = accept(&addr, stream, |req| async {
                    handler(req).await
                }).await {
                    eprintln!("{:?}", err);
                }
            });
        }
        Ok(())
    }
}

