
#![allow(unused_variables,unused_imports)]
#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
use std::fs::File;
use serde::{Serialize,Deserialize};
use serde_yaml;
use std::path::{PathBuf,Path};
use notify::{Watcher, RecursiveMode, watcher,DebouncedEvent,RawEvent};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use kuchiki::traits::*;
use std::sync::{Arc, Mutex};
use std::env;

#[derive(Hash)]
#[derive(Debug,Eq, PartialEq, Serialize, Deserialize,Clone)]
pub enum Method {
    GET,
    POST
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        //write!(f, "{:?}", self)
        // or, alternatively:
        std::fmt::Debug::fmt(self, f)
    }
}


#[derive(Debug, PartialEq, Serialize, Deserialize,Clone)]
pub struct Component {
  pub id: String,
  pub css_selector: String,
  pub template_file: String,
  pub call: String,
  pub rpc: Option<RPC>
}

#[derive(Hash)]
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize,Clone)]
pub struct RPC{
    pub uri:String,
    pub method: Method,
    pub body:Option<String>,
    pub headers: Vec<String>,
    pub timeout:u16,
    pub ttl:u16
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize,Clone,Default)]
pub struct ServerConfig{
    pub project_root: String, 
    pub sites_root:  String,
    pub workers: u16,
    pub bind: String,
    pub port: String,
    pub default_site: String
}


#[derive(Debug, PartialEq, Serialize, Deserialize,Clone)]
pub struct Components{
  pub components: Vec<Component>
}


#[derive(Debug, PartialEq, Serialize, Deserialize,Clone)]
pub struct Contents{
  pub contents: Vec<Content>
}


#[derive(Debug, PartialEq, Serialize, Deserialize,Clone)]
pub struct Content{
  pub id: String,
  pub content: HashMap<String,String>
}


pub trait ToRegistryFrom<S,R>{
    fn fromTo(source:S,to:R);
}


#[derive(Debug, PartialEq, Serialize, Deserialize,Clone)]
pub struct Rules{
  pub rules: Vec<Rule>
}


#[derive(Debug, PartialEq, Serialize, Deserialize,Clone)]
pub struct Rule {
  pub id: String,
  pub expr: String
}

impl ToRegistryFrom<File,&mut Registry> for Components{
    fn fromTo(f: File,r:&mut Registry){
         let res = serde_yaml::from_reader::<_,Self>(f);
         match res{
           Ok(d) => {
            println!("Pushing Component to registry");
            for c in d.components{
                let d = c.clone();
                r.compMap.insert(d.id, c);
            }
           },
           Err(e) => {
               println!("Object Component could not be loaded {:?} ",e);
           }  
         }
    }
}

impl ToRegistryFrom<File,&mut Registry> for Rules{
    fn fromTo(f: File,r:&mut Registry){
         let res = serde_yaml::from_reader::<_,Self>(f);
         match res{
           Ok(d) => {
            println!("Pushing Rules to registry");
            for c in d.rules{
                let d = c.clone();
                r.ruleMap.insert(d.id, c);
            }
            },
           Err(e) => {
               println!("Object Rule could not be loaded {:?}",e);
           }  
         }
    }
}

impl ToRegistryFrom<File,&mut Registry> for Contents{
    fn fromTo(f: File,r:&mut Registry){
        let res = serde_yaml::from_reader::<_,Self>(f);
        match res{
            Ok(d) => {
             println!("Pushing Content to registry");
             for c in d.contents{
                 let d = c.clone();   
                 r.contents.insert(d.id, c);
             }
             },
            Err(e) => {
                println!("Object Content could not be loaded {:?}",e);
            }  
          }

    }
}



impl Default for Component {
    fn default() -> Component {
        Component{id:"".to_string(),css_selector:"".to_string(),template_file:"".to_string(),call:"".to_string(),rpc:None}
    } 
}

impl Default for Rule{
    fn default() -> Rule {
        Rule{id:"".to_string(),expr:"".to_string()}
    }

}

pub struct Registry{
    pub id: String,
    pub init_path:String,
    pub compMap:HashMap<String,Component>,
    pub ruleMap:HashMap<String,Rule>,
    pub templates:HashMap<String,Vec<u8>>,
    pub contents:HashMap<String,Content>
}

pub trait Loader{
    fn load(path:PathBuf,this:&mut Self);
}

impl Default for Registry{
    fn default() -> Registry{
        Registry{
            
            id:"default".to_string(),
            init_path:"".to_string(),
            compMap:HashMap::new(),
            ruleMap:HashMap::new(),
            templates:HashMap::new(),
            contents:HashMap::new()
        }
    }
}

impl Loader for Registry{
  

    fn load(path:PathBuf,this:&mut Self){
        match path.to_str() {
            Some(str)=>{
             
                if str.contains("components.yaml"){
                    let f = File::open(path.clone()).unwrap();
                    Components::fromTo(f,this);  
      
                }

                if str.contains("rewrites.yaml"){
                    let f = File::open(path.clone()).unwrap();
                    Rules::fromTo(f,this);              
                }

                if str.contains("content.yaml"){
                    let f = File::open(path.clone()).unwrap();
                    Contents::fromTo(f,this);              
                }

                if str.ends_with("jinja"){
                    let mut f = File::open(path.clone()).unwrap();
                    let mut contentu8:Vec<u8>  = Vec::new();
                    f.read_to_end(&mut contentu8).expect("Unable to read"); 
                    //fix code here
                    match path.strip_prefix(this.init_path.clone()){
                        Ok(key)=>{
                            println!("inserting template {:?}",key);
                            this.templates.insert(String::from(key.to_string_lossy()), contentu8);   
                        },
                        Err(e)=>{
                            println!("issue loading template {:?} {:?}",e,path);
                        }


                    };
                }



            },
            None => {

            }
        }
    }
}
