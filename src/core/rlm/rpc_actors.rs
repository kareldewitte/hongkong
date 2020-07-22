pub mod rpc_actors{
     
    use actix_web::client::{Client,ClientResponse,ClientRequest,SendRequestError};
    use actix_web::web;
    use actix_web::Error;
    use actix_http::body::{Body, BodySize, MessageBody, ResponseBody, SizedStream};
    use serde::{Serialize,Deserialize};
    use std::collections::{HashMap};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use std::sync::{Arc, Mutex};
    use actix_web_actors::HttpContext;
    use futures::future::{ok, err};
    use actix::{Actor,Handler,Message,Context};
    use actix::prelude::*;
    use crate::core::rlm::parser::{RPC,Method};
    use ureq;
    use mime;
    use std::io::Cursor;
    use ttl_cache::TtlCache;


    #[derive(Debug,Eq, PartialEq, Serialize, Deserialize,Clone)]
    #[derive(Hash)]
    pub struct SendRequest{
       pub rpc:RPC
    }

    
    pub struct RpcExecutor{
        pub component_cache:TtlCache<SendRequest, String>,
        //pub page_cache:TtlCache<SendRequest,Response>
    }

    pub trait utils{
        fn setHeaders(&mut self,rpc:&RPC)->&mut Self;
        fn setAuth(&mut self,rpc:&RPC)->&mut Self;
    }

    impl utils for ureq::Request{
        fn setHeaders(&mut self, rpc: &RPC)->&mut Self{
            for head in &rpc.headers{
                let h:Vec<&str> = head.split(":").collect();
                self.set(h[0],h[1]);
            }
            self
        }
        fn setAuth(&mut self,rpc: &RPC)->&mut Self{
            match &rpc.auth{
                Some(auth)=>self,
                None=>self
            }
        }
    }
    
    
    impl Message for SendRequest {
        type Result = Result<Mutex<String>, std::io::Error>;
    }
    

    impl Actor for RpcExecutor{
            type Context = SyncContext<Self>;
    }



    fn call(rpc:&RPC)-> ureq::Response{
        
        let resp = match &rpc.method{
            Method::GET => {
                
                ureq::get(&rpc.uri)
                .setHeaders(&rpc)
                .timeout_connect(rpc.timeout.into())
                .call()
                
            },
            Method::POST => {
                match &rpc.body{
                    Some(b)=>{
                        
                        let read = Cursor::new(b.clone().into_bytes());
                        ureq::post(&rpc.uri)
                        .setHeaders(&rpc)
                        .timeout_connect(rpc.timeout.into())
                        .send(read)
                    },
                    None=>{
                        ureq::Response::new(500, &"Configuration error", &"Configuration error: No body in post request")
                    }
                }
               

            }                
        };            
       
        resp

    }

    
    /// Define handler for `Ping` message
    impl Handler<SendRequest> for RpcExecutor {
        type Result = Result<Mutex<String>, std::io::Error>;
        fn handle(&mut self, msg: SendRequest, _: &mut Self::Context) -> Self::Result {
            println!("begin blocking");
            let rpc = &msg.rpc;
            let mut body = String::default();
            //let kvs: Vec<_> = self.cache.iter().collect();
            //println!("misses {:?}",self.cache.);
            body = match self.component_cache.get(&msg){
                Some(resp)=>{
                    println!("Found for sendrequest {:?}",resp);
                    resp.to_string()
                },
                None=> {

                    let mut b = String::default();
                    let resp = call(rpc);
                    if resp.ok() {
                        b = resp.into_string().unwrap();
                        if(rpc.ttl>0){
                            self.component_cache.insert(msg.clone(),b.clone(), Duration::from_secs(rpc.ttl.into()));
                        }
                      } else {
                        b = resp.status().to_string()+resp.status_line()
                    }
                               
                    b
                }
            };            
            
            //println!("Ping received");
            Ok(Mutex::new(body))
        }
    }

}