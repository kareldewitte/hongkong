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
use std::str;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

unsafe impl Send for Cache{}

                         

#[derive(Debug, PartialEq, Clone)]
pub struct Page{
    pub filepath:String,
    pub content:Vec<u8>,
    pub hash:u64,
    pub node: kuchiki::NodeRef
}


#[derive(Debug, PartialEq, Clone)]
pub struct Cache{
  pub pages: HashMap<String,Page>
}



pub trait CacheFunctions<S>{
    fn getUpdate(&mut self,source:S) -> Result<Page,&str>;    
    fn size(&mut self)->usize;
}

impl Default for Cache{
    fn default() -> Cache {
        Cache{
            pages:HashMap::new()
        }
    }
}

impl CacheFunctions<String> for Cache{
    fn size(&mut self)->usize{
        self.pages.len()
    }

    fn getUpdate(&mut self,source:String) -> Result<Page,&str>{

        let src = source.clone();
        let pbuf = PathBuf::from(src.clone());
        if !pbuf.exists() {
            return Err("File does not exist");
        }
        let mut file_content:Vec<u8> = Vec::new();
        match File::open(source){
            Ok(mut file) => {
                match file.read_to_end(&mut file_content){
                    Ok(size) => {
                        let contents = str::from_utf8(&mut file_content).unwrap().to_string();
                        let mut hasher = DefaultHasher::new();
                        // write input message
                        file_content.hash(&mut hasher);
                        let hash = hasher.finish();
                        //check if page exists
                        match self.pages.get(&src.to_string())
                        {
                            Some(c) => {
                                //println!("hash {:?},{:?}",c.hash,hash);
                                if c.hash == hash{
                                    //println!("dont recalc doc exists");
                                    Ok(c.clone())
                                } else {
                                    //update
                                    //println!("update");
                                    let mod_content = contents.clone();
                                    let document = kuchiki::parse_html().one(mod_content);
                                    let page =Page{filepath: src.to_string(),content:file_content,hash:hash,node:document};                    
                                    self.pages.insert(src.to_string(),page.clone());
                                    Ok(page)    
                                }
                            },
                            None => {
                                //insertupdate
                                //println!("insert update");
                                let mod_content = contents.clone();
                                let document = kuchiki::parse_html().one(mod_content);
                                let page =Page{filepath: src.to_string(),content:file_content,hash:hash,node:document};                    
                                self.pages.insert(src.to_string(),page.clone());
                                Ok(page)
                            }
                        }
                    
                    },
                    Err(e)=>{
                        Err("")
                    }
                }
            },
            Err(e)=>{
                Err("")
            }
        }  
    }     
}