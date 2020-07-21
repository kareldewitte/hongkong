pub mod exposer { 
    
    use std::fs::File;
    use serde_yaml;
    use std::path::{PathBuf,Path};
    use crate::core::rlm::parser::{Registry,Loader,Components,Component,Rules,Rule};
    use notify::{Watcher, RecursiveMode, watcher,DebouncedEvent,RawEvent};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

        

    pub fn init(file_name:String) -> Result<(Arc<Mutex<Registry>>), Box<std::error::Error>>{
        
        let (tx, rx) = channel();
        let mut registry = Registry::default();
        registry.init_path = file_name.clone();
        println!("Init path :{:?}",registry.init_path);
        let regmutex: Arc<Mutex<Registry>> = Arc::new(Mutex::new(registry));
        let regmutex_c = Arc::clone(&regmutex);  
        let fname = file_name.clone();
        //initial load
        std::thread::spawn(move || {
                let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
                match watcher.watch(file_name, RecursiveMode::Recursive){
                    Ok(success) => {

                        loop{
                            match rx.recv() {
                                Ok(DebouncedEvent::Write(path)) => {
                                    match path.to_str() {
                                       Some(str)=>{
        
                                           let mut reg = regmutex_c.lock().unwrap();
                                           Registry::load(path, &mut reg);
                                            println!("Number of items in compmap: {:?}",reg.compMap.len());
                                            println!("Number of items in rulemap: {:?}",reg.ruleMap.len());
                                           
                                       },
                                       None => {
        
                                       }
                                    }
                                },
                                Ok(event) => {
                                    //println!("broken event: {:?}", event)
                                },
        
                                Err(e) => {
                                    //println!("watch error: {:?}", e)
                                },
                            } 
                        }
                    },
                    Err(e)=>{
                        println!("Problem loading the watcher {:?} {:?}",e,fname);
                    }

                };
                
        });
        Ok(Arc::clone(&regmutex))
    }

}
