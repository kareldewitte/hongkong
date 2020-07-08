#![allow(unused_variables, unused_imports)]
#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

pub mod core;

use actix::*;
use actix_files as afs;
use actix_rt;
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::future::*;
use futures::future::{ready, Ready};
use kuchiki::traits::*;
use kuchiki::{ElementData, NodeDataRef, NodeRef};
use log::Level;
use log::{error, info, warn};
use mime_guess;
use reqwest::Client;
use rug::{Assign, Integer};
use sedregex::{find_and_replace, ReplaceCommand};
use serde::Serialize;
use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

static sitesRoot: &str = "/home/karel/Projects/rustpjcts/test/proxy_01/src/resources/sites/";

use crate::core::middle::transformer::{build_context, Calls, WebContext};
use crate::core::rlm::cache::{Cache, CacheFunctions, Page};
use crate::core::rlm::parser::{Component, Registry,ServerConfig};
use crate::core::rlm::rloader::exposer;
use crate::core::rlm::rpc_actors::rpc_actors::{RpcExecutor, SendRequest};
use actix::prelude::*;


#[derive(Serialize)]
struct WebPage {
    content: Vec<u8>,
    mimeType: String,
}

struct AppState {
    registry: Arc<Mutex<Registry>>,
    pagecache: Arc<Mutex<Cache>>,
    rpcexec: Addr<RpcExecutor>,
    //httpclient: Client
    servconf: Arc<Mutex<ServerConfig>>,
    httpclient: Arc<Mutex<Client>>,
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

fn touchFiles(dir: &str) {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir() && !e.file_name().to_str().unwrap().contains("sites"))
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&entry.path())
            .unwrap();
        let space = '.';
        let cr = '\r';
        let mut mess: Vec<u8> = Vec::new();
        mess.push('\r' as u8);
        file.write(&mess);
        //println!("touching {:?}",entry);
    }
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}


async fn render(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let host: &str = match req.headers().get("x-site") {
        Some(host) => host.to_str().unwrap(),
        None => "default",
    };

    let servconf = (*data.servconf.deref()).lock().unwrap();
    let postfix = host.to_owned() + "/" + req.match_info().get("path").unwrap_or("index.html");
    let path = format!("{}{}", servconf.sites_root, postfix);
    //println!("requested {:?}",&path);
    //info!{"path {}",path};
    let pathbuf: std::path::PathBuf = PathBuf::from(path.clone());
    let mime = mime_guess::from_path(pathbuf)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    //error!{"path {} mime {}",path,mime};
    let reg = (*data.registry.deref()).lock().unwrap();
    let rpcexec = &data.rpcexec;
    
    let client = (*data.httpclient.deref()).lock().unwrap();
    //let client = &data.httpclient;
    //println!("data {:?}",reg.compMap.len());
    let mut pagecache = (*data.pagecache.deref()).lock().unwrap();
    let mut contentu8: Vec<u8> = read_file(path.clone());

    //let httpclientMutex = (*data.httpclient.deref()).lock().unwrap();
    //let httpclient = httpclientMutex.deref();
    if contentu8 != "None".as_bytes() {
        if mime == "text/html".to_string() {
            let now = Instant::now();
            println!("size {:?}",pagecache.size());
            match pagecache.getUpdate(path.clone()) {
                Ok(page) => {
                    let document = page.node;
                    for comp in reg.compMap.clone().values() {

                        let css_selector = comp.css_selector.clone();
                        let matches = document.select(&css_selector);
                        // build request and execute request here
                        let mut wb: WebContext = build_context(&req);
                        // perform query
                        let now = Instant::now();
                        let _ = match comp.get_rpc(&wb) {
                            Ok(req) => {
                                let bodymut = rpcexec.send(req).await.unwrap().unwrap();
                                let mm = bodymut.lock().unwrap();
                                wb.resp = Some(mm.clone());
                                drop(mm);
                            }
                            Err(e) => {
                                println!("Message {:?}", e);
                            }
                        };
                        println!("request executed in {}mus", now.elapsed().as_micros());

                        match matches {
                            Ok(m) => {
                                let mut counter = 0;
                                let mut nodevec: Vec<NodeDataRef<ElementData>> = Vec::new();
                                // we need a copy here because tree will be altered by detach
                                for i in m {
                                    nodevec.push(i);
                                }
                                for css_match in nodevec {
                                    //println!("Found for {:?}:{:?}", css_selector, counter);
                                    let as_node = css_match.as_node();
                                    let result = match &comp.call[..] {
                                        "remove" => comp.remove(as_node),
                                        "replace_and_render" => {
                                            comp.replace_and_render(as_node, &reg, req.clone(), &wb)
                                        }
                                        _ => println!("No function {:?} available", comp.call),
                                    };
                                    counter = counter + 1;
                                }
                            }
                            Err(e) => {}
                        };
                        //println!("Ending ");
                    }
                    let mut doc_content: String = document.to_string();

                    let mut regvec: Vec<&str> =
                        reg.ruleMap.values().map(|e| e.expr.as_str()).collect();

                    let mut contents: String = match find_and_replace(&doc_content, regvec) {
                        Ok(res) => res.to_string(),
                        Err(e) => {
                            println!("Error {:?}", e);
                            doc_content
                        }
                    };

                    contentu8 = contents.into_bytes();
                }
                Err(reason) => {
                    println!("{:?}", reason);
                }
            }
            info! {"path {}",path};
        }
    }

    WebPage {
        content: contentu8,
        mimeType: mime,
    }
}

async fn serve(req: HttpRequest) -> Result<afs::NamedFile, Error> {
    let path: std::path::PathBuf = PathBuf::from(
        sitesRoot.to_owned() + "/" + req.match_info().get("path").unwrap_or("index.html"),
    );
    let file = afs::NamedFile::open(path)?;
    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}

//#[actix_rt::main]
fn main() {
    //-> std::io::Result<()> {
    env_logger::init();
    let mut server_config = ServerConfig::default();
    let args: Vec<String> = env::args().collect();
    if(args.len()>1){
        //println!("{:?}",args[1]);
        let mut file = fs::File::open(&(args[1].clone()+"/config.yaml")).expect("Unable to open file");
        server_config = serde_yaml::from_reader::<_,ServerConfig>(file).unwrap();
        //println!("{:?}",server_config);

    }else{
        println!("Please specify server config directory !");
        return
    }
    
    


  
    //pages/main.yaml

    let regmutex: Arc<Mutex<Registry>> = match exposer::init(server_config.project_root.clone()) {
        Ok(rtmx) => {
            Arc::clone(&rtmx)
            //Arc::new(Mutex::new(Registry::default()))
        }
        Err(_err) => {
            println!("Failed to load registry");
            Arc::new(Mutex::new(Registry::default()))
        }
    };

    let sys = actix::System::new("rpc-executors");

    // Start 3 parallel db executors
    let addr = SyncArbiter::start(1, || RpcExecutor {
        component_cache: ttl_cache::TtlCache::new(1000)
    });

    let srv_conf = Arc::new(Mutex::new(server_config.clone()));

    let cache = Arc::new(Mutex::new(Cache::default()));
    //let deref = unsafe { ptr.as_ref() };
    let client = Arc::new(Mutex::new(Client::new()));
    //let client = Client::new();
    let state = web::Data::new(AppState {
        registry: regmutex,
        pagecache: cache,
        rpcexec: addr.clone(),
        httpclient: client,
        servconf:srv_conf
    });

    println!("Initialising server");
    std::thread::sleep(Duration::from_millis(500));
    touchFiles(&(server_config.project_root.clone() + &"/commons".to_string()));
    touchFiles(&(server_config.project_root.clone() + &"/content".to_string()));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(greet))
            .route("/{path:.*}", web::get().to(render))
    })
    .workers(server_config.workers.into())
    .bind(server_config.clone().bind.clone())
    .unwrap()
    .run();
    //.unwrap();
    println!("Started http server at {:?}:",server_config.bind.clone());
    let _ = sys.run();
}
