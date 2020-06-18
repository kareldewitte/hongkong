#![allow(unused_variables,unused_imports)]
#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

pub mod core;

use actix_web::{web, App, HttpRequest, HttpServer, HttpResponse, Error, Responder};
use std::time::{Duration, Instant};
use rug::{Assign, Integer};
use std::env;
use std::fs;
use std::path::{PathBuf,Path};
use serde::Serialize;
use futures::future::{ready, Ready};
use log::{info, warn, error};
use log::Level;
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_files as afs;
use std::str;
use std::io;
use std::io::prelude::*;
use walkdir::WalkDir;
use mime_guess;
use std::sync::{Arc, Mutex};
use std::ops::Deref;
use kuchiki::traits::*;
use kuchiki::{NodeDataRef,NodeRef,ElementData};
use sedregex::{find_and_replace, ReplaceCommand};
use reqwest::{Client,blocking};


static sitesRoot:&str = "/home/karel/Projects/rustpjcts/test/proxy_01/src/resources/sites/";


use crate::core::rlm::rloader::{exposer};
use crate::core::rlm::parser::{Registry,Component};
use crate::core::middle::transformer::{Calls};
use crate::core::rlm::cache::{Cache,Page,CacheFunctions};


#[derive(Serialize)]
struct WebPage {
    content: Vec<u8>,
    mimeType: String,
}


struct AppState {
    registry: Arc<Mutex<Registry>>,
    pagecache: Arc<Mutex<Cache>>,
    httpclient: Arc<Mutex<Client>>
}

// Responder
impl Responder for WebPage {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;
    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        
        let body = self.content;
        // Create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type(self.mimeType)
            .body(body)))
    }
}

fn read_file(mut file_name: String) -> Vec<u8> {
    if file_name.is_empty() {
        file_name = String::from("index.html");
    }

    let path = Path::new(&file_name);
    if !path.exists() {
        return String::from("None").into();
    }
    let mut file_content = Vec::new();
    let mut file = fs::File::open(&file_name).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");
    file_content
}


fn touchFiles(dir:&str){
   
    for entry in WalkDir::new(dir)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| !e.file_type().is_dir() && !e.file_name().to_str().unwrap().contains("sites")) {
        let mut file = fs::OpenOptions::new().write(true).append(true).open(&entry.path()).unwrap();
        let space = '.';
        let cr = '\r';
        let mut mess:Vec<u8> = Vec::new();
        mess.push('\r' as u8); 
        file.write(&mess);
        //println!("touching {:?}",entry);
    }
}



async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}



async fn render(req: HttpRequest,data: web::Data<AppState>)-> impl Responder {
    
    let host:&str = match req.headers().get("x-site"){
        Some(host) => {
            host.to_str().unwrap()
        },
        None => {
            "default"
        }
    };

    let postfix = host.to_owned()+"/"+req.match_info().get("path").unwrap_or("index.html"); 
    let path = format!("{}{}",sitesRoot,postfix);
    //println!("requested {:?}",&path);
    
    //info!{"path {}",path};
    let pathbuf: std::path::PathBuf = PathBuf::from(path.clone());
    let mime = mime_guess::from_path(pathbuf).first_or_octet_stream().essence_str().to_string();
    //error!{"path {} mime {}",path,mime};
    let reg = (*data.registry.deref()).lock().unwrap();
    //println!("data {:?}",reg.compMap.len());
    let mut pagecache = (*data.pagecache.deref()).lock().unwrap();
    let mut contentu8:Vec<u8> = read_file(path.clone());   
    let httpclientMutex = (*data.httpclient.deref()).lock().unwrap();
    let httpclient = httpclientMutex.deref();
    if contentu8 != "None".as_bytes() {    
        if mime == "text/html".to_string() { 
            let now = Instant::now();
            match pagecache.getUpdate(path.clone()){
               Ok(page)=>{
                let document = page.node;
                    for comp in reg.compMap.clone().values() {
                        let css_selector = comp.css_selector.clone();
                        let matches = document.select(&css_selector);
                        match matches{
                            Ok(m)=>{
                                let mut counter = 0;
                                let mut nodevec:Vec<NodeDataRef<ElementData>> = Vec::new();
                                // we need a copy here because tree will be altered by detach
                                for i in m{ nodevec.push(i);};
                                for css_match in nodevec {  
                                    println!("Found for {:?}:{:?}",css_selector,counter);
                                    let as_node = css_match.as_node();
                                    let result = match &comp.call[..] {
                                        "remove" => {comp.remove(as_node)},
                                        "replace_and_render" => {comp.replace_and_render(as_node,&reg,req.clone(),httpclient)}
                                        _ => {println!("No function {:?} available",comp.call)},
                                    };  
                                    counter = counter+1; 
                                } 
                            },
                            Err(e) =>{
                                
                            }
                        };
                        //println!("Ending ");                        
                        
                    }
                    let mut doc_content:String = document.to_string();
                    //println!("Doc {:?}",doc_content);
                    let mut regvec:Vec<&str> = reg.ruleMap.values().map(
                        |e| 
                         e.expr.as_str()
                    ).collect();
                    
                    let mut contents:String = match find_and_replace(&doc_content, regvec){
                        Ok(res)=>{
                            res.to_string()
                        },
                        Err(e)=>{
                            println!("Error {:?}",e);
                            doc_content
                        }                        
                    };
                    //let mut contents = find_and_replace(&doc_content, &[r"s/https:\/\/www.rochefoundationmedicine.com\//\//ig"]).unwrap().to_string();
                    contentu8 = contents.into_bytes();
               },
               Err(reason)=>{
                    println!("{:?}",reason);
               } 
            }
            info!{"path {}",path};
        }
    }   
    
    WebPage { content: contentu8, mimeType: mime}
   
}




async fn fact(req: HttpRequest) -> impl Responder {
    let n = req.match_info().get("n").unwrap_or("5");
    let ni: i32 = n.parse().unwrap();
    let mut a: i32 = ni;
    let mut r = Integer::from(1);
    let now = Instant::now();

    while a>1 {
        r = r*a;
        a=a-1;
    }
    format!("{}!=>{} in {}ms",&n, &r,now.elapsed().as_micros())

}


async fn serve(req: HttpRequest) -> Result<afs::NamedFile, Error> {
    let path: std::path::PathBuf = PathBuf::from(sitesRoot.to_owned()+"/"+req.match_info().get("path").unwrap_or("index.html"));
    
    let file = afs::NamedFile::open(path)?;
    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let resources = "/home/karel/Projects/rustpjcts/test/proxy_01/src/resources";
    //pages/main.yaml
    let reqw_client = Arc::new(Mutex::new(Client::new()));    
    let regmutex: Arc<Mutex<Registry>> = match exposer::init(resources.to_string()){
        Ok(rtmx)=>{
             Arc::clone(&rtmx)
             //Arc::new(Mutex::new(Registry::default()))  
        },
        Err(_err)=>{
            println!("Failed to load registry");
            Arc::new(Mutex::new(Registry::default()))
        },
    };
    
    let cache = Arc::new(Mutex::new(Cache::default()));
    //let deref = unsafe { ptr.as_ref() };
    let state =  web::Data::new(AppState{registry:regmutex,pagecache:cache,httpclient:reqw_client});
    println!("Initialising server");
    std::thread::sleep(Duration::from_millis(500));
    touchFiles(&(resources.to_string()+&"/commons".to_string()));
    touchFiles(&(resources.to_string()+&"/content".to_string()));

    HttpServer::new(move || 
    {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(greet))
            .route("/{path:.*}", web::get().to(render))
            .route("/fact/{n}",web::get().to(fact))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}