use std::pin::Pin;
use std::cell::RefCell;
use actix_service::{Service, Transform};
use actix_http::http::{HeaderMap};
use actix_web::{web,dev::ServiceRequest, dev::ServiceResponse, Error,HttpRequest};
use futures::future::{ok, Ready};
//use futures::Future;

use std::sync::{Arc, Mutex};
use serde::{Serialize};
use serde_json::{Value};
use std::str;
use std::io;
use std::io::prelude::*;
use actix::prelude::*;

use tera::{Tera,Context,Result};
use crate::core::rlm::rloader::{exposer};
use crate::core::rlm::parser::{Component,Registry,Content};
use crate::core::rlm::rpc_actors::rpc_actors::{RpcExecutor,SendRequest};
use kuchiki::traits::*;
use kuchiki::{Doctype, DocumentData, ElementData, Node, NodeData, NodeRef};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use futures::executor::block_on;
//use actix_web::client::{Client,ClientResponse,ClientRequest};
use reqwest::{Client,Request,RequestBuilder};
//use actix_web::http::*;
//use mime;
//use crate::core::rlm::rpc_actors::rpc_actors;
//use actix_rt::System;
//use tokio::{spawn};
use futures::future::*;
use reqwest::header::{HeaderValue, USER_AGENT, CONTENT_TYPE};
use std::ops::Deref;
use actix::Addr;




#[derive(Serialize)]
pub struct WebContext<'a>{
    path:&'a str,
    params:HashMap<&'a str,Vec<&'a str>>,
    //cookies:&'a str
    //headers:HeaderMap
    pub resp:Option<String>,
    pub json_object:Option<HashMap<String, serde_json::Value>>,
}

fn from_param(values:&str)-> Vec<&str>{
    let mut vv:Vec<&str> = Vec::new();
    if values.contains(","){
        let pps:Vec<&str> = values.split(",").clone().collect();
        vv.extend(&pps);
        
    }else{
        vv.push(values);
    }
    vv
}

pub fn build_context<'a>(req:&'a HttpRequest)-> WebContext<'a>{
    let mut params:HashMap<&'a str,Vec<&'a str>> = HashMap::new();
    let q = req.query_string();
    if q!= "" {
     for kp in q.split("&"){
        let kv:Vec<&str> = kp.split("=").clone().collect();
        match params.get(kv[0]){
            Some(e)=>{
                //check if kv[1] is in form of ,
                e.clone().extend(from_param(kv[1]));
                params.insert(kv[0].clone(), e.to_vec());
            },
            None=>{
                params.insert(kv[0].clone(), from_param(kv[1]));
            }
        }
     }
    }
    WebContext{
        path:req.path(),
        params:params,
        resp:None,
        json_object:None,
    }
}



pub trait replace<T,R>{
    fn replace(t:T) -> R; 
}


pub trait Calls{
    
    fn replace_and_render(&self,t:&NodeRef,r:&Registry,req: HttpRequest, wb: &WebContext);
    //fn replace_and_render_remote(&self,t:&NodeRef,r:&Registry);
    fn remove(&self,t:&NodeRef); 
    fn get_content(&self,t:&NodeRef,r:&Registry)->Option<Content>;
    fn get_rpc(&self,wb:&WebContext,content:Option<Content>)->Result<SendRequest>;
}

trait Inner{
    fn replace(&self,r:Result<String>,t:&NodeRef);
}


impl Inner for Component{

    fn replace(&self,result:Result<String>,t:&NodeRef) -> () {
        match result {
            Ok(e)=>{
                //let h = htmlescape::decode_html(&e).unwrap();
                let nref = kuchiki::parse_html().one(e);
                //let parent:NodeRef = t.parent().unwrap();
                let mut children = true;
                while children {
                    match t.first_child(){
                        Some(c)=>{
                            c.detach();
                        },
                        None => {
                            children=false;
                        }
                    }
                }
                t.append(nref);
            },
            Err(e)=>{
                println!("Problem replacing {:?}",e);
            }
        }; 

    }

}



impl Calls for Component{

    fn get_rpc(&self,wb: &WebContext,content:Option<Content>)->Result<SendRequest>{
        let mut context = Context::new();
        context.insert("webcontext",&wb);  
        match content{
            Some(e)=> context.insert("rep",&e),
            None => {}//println!("No content for {:?}",&self.rpc)
        };  
        match &self.rpc{
            Some(rpc)=>{
                match Tera::one_off(&rpc.uri,&context, true){
                    Ok(uri)=>{
                        //println!("rendered uri {:?}",uri);
                        let mut rpcs = rpc.clone();
                        rpcs.uri = uri;
                        let sr = SendRequest{rpc:rpcs};
                        Ok(sr)   
                    },
                    Err(e) =>{
                     Err(e)   
                    }
                }
            },
            None=>{
                Err(tera::Error::msg("No RPC url"))
            }
        }
    }

    fn get_content(&self,t: &NodeRef, r: &Registry)->Option<Content>{
               
        let attr = t.as_element().unwrap().attributes.borrow();
        let tdata = attr.get("data-content-id");
        //println!("found {:?}",tdata);
        match tdata{
            Some(e) => {
                match r.contents.get(e){
                    Some(co)=>{
                        Some(co.clone())
                    },
                    None => None
                }
            }
            ,
            None => None
        }
            
    }


    fn replace_and_render(&self,t:&NodeRef,r:&Registry,req: HttpRequest, wb: &WebContext){
        let now = Instant::now();
        //do null check
        // let u8templ:&Vec<u8> = match r.templates.get(&self.template_file){
        //     Some(e)=>{
        //         e
        //     },
        //     None=>{
        //        return;
        //     }
        // };
        
        //let template = str::from_utf8(&u8templ.clone()).unwrap().to_string();
        let mut context = Context::new();
        
        match self.get_content(t,r){
                    Some(e) => {
                        //println!("{:?}",e);
                        context.insert("rep",&e);
                        context.insert("webcontext",&wb);
                        //execute request in here
                        let result = r.tera.render(&self.template_file, &context);
                        self.replace(result,t);   
                        
                    },
                    None => {
                        context.insert("ctx", "bla");
                    }
                }
             

       
       
    }




    fn remove(&self,t: &NodeRef){
        t.detach();        
    }
}